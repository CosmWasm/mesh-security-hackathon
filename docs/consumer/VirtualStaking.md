# Virtual Staking

Virtual Staking is a permissioned contract on the Consumer side that can interact with the
native staking module in special ways. There are usually multiple Virtual Staking contracts
on one Consumer chain, one for each active Provider, linked 1-to-1 with a Converter.

## Previous Work

There is a lot of overlap with this design and Osmosis' [design of SuperFluid Staking](https://github.com/osmosis-labs/osmosis/tree/main/x/superfluid).
Before commenting on the technical feasibility or implementation on the SDK side, it would be good to review this in depth.
This is probably the biggest change made to staking functionality without forking `x/staking` and should be leveraged for the MVP at least.

## Interface

The entry point to the Virtual Staking system is a contract that can be called by one Converter, and which
has some special abilities to stake virtual tokens. We cannot let any receiver mint arbitrary tokens, or we lose all security,
so each "Converter / Virtual Staking Contract" pair has permission of a maximum amount of "virtual stake" that it can provide
to the system. Anything over that is ignored, after which point, the average rewards per cross-staker start to diminish as
they split a limited resource.

The `Converter` should be able to call into the `Virtual Staking` contract with the following:

```rust
pub enum VirtualStakeExecMsg {
  /// This mints "virtual stake" if possible and bonds to this validator.
  Bond {
    amount: Uint128,
    validator: String,
  },
  /// This unbonds immediately, not like standard staking Undelegate
  Unbond {
    amount: Uint128,
    validator: String,
  },
}
```

The `Converter` should be able to query the following info from the contract:

```rust
pub enum VirtualStakeQueryMsg {
  #[returns(BondStatusResponse)]
  BondStatus { },
}

pub struct BondStatusResponse {
  pub cap: Uint128,
  pub delegated: Uint128,
}
```

Finally, the virtual staking contract should make the following call into the `Converter`,
which is send along with a number of `info.funds` in the native staking token:

```rust
pub enum ConverterExecMsg {
  /// This is required, one message per validator all info.funds go to those delegators
  DistributeRewards {
      validator: String,
  },
  /// Optional (in v1?) to optimize this, by sending multiple payments at once.
  /// info.funds should equal rewards.map(|x| x.reward).sum()
  DistributeRewardsMulti {
    rewards: Vec<RewardInfo>,
  },
}

pub struct RewardInfo {
  pub validator: String,
  pub reward: Uint128,
}
```

## Extensibility

Virtual Staking is primarily defined by the interface exposed to the Convert contract, so we can use wildly
different implementations of it on different chains. Some possible implementations:

* A CosmWasm contract with much of the logic, calling into a few custom SDK functions.
* A CosmWasm contract that just calls into a native module for all logic.
* An entry point to a precompile (eg. Composable uses "magic addresses" to call into the system, rather than `CustomMsg`)
* A mock contract that doesn't even stake (for testing)

All these implementations would require a Virtual Staking contract that fulfills the interface above and is informed by the
specific design discussed here, but it can by creative in the implementation.

The rest of this document describes a "standard SDK implementation" that we will ship with Mesh Security.
Since we want this to be portable and likely to be integrated into as many chains as possible, we will look
for a design minimizing the changes needed in the Go layer, and focus on keeping most logic on CosmWasm.
We can add other implementations later as needed.

## Standard Implementation

Virtual Staking will be a mix of a Cosmos SDK module and a CosmWasm contract. 
We need to expose some special functionality through the SDK Module and `CustomMsg` type.
The module will contain a list of which contract has what limit, which can only be updated
by the native governance. The interface is limited and can best be described as Rust types:

```rust
#[cw_serde]
pub enum CustomMsg {
  /// Embed one level here, so we are independent of other custom messages (like TokenFactory, etc) 
  VirtualStake(VirtualStakeMsg),
}

#[cw_serde]
/// These are the functionality 
pub enum VirtualStakeMsg {
  /// This mints "virtual stake" if possible and bonds to this validator.
  Bond {
    amount: Uint128,
    validator: String,
  },
  /// This unbonds immediately, not like standard staking Undelegate
  Unbond {
    amount: Uint128,
    validator: String,
  },
}
```

It will also need a query to get its max cap limit.  Note that it can use standard `StakingQuery` types to query it's existing delegations
as they will appear as normal in the `x/staking` system.

```rust
#[cw_serde]
pub enum CustomQuery {
  /// Embed one level here, so we are independent of other custom messages (like TokenFactory, etc) 
  VirtualStake(VirtualStakeQuery),
}

#[cw_serde]
/// These are the functionality 
pub enum VirtualStakeQuery {
  #[returns(MaxCapResponse)]
  MaxCap { },
}

pub struct MaxCapResponse {
  pub cap: Uint128,
}
```

### Contract

Each Virtual Staking contract is deployed with a Converter contract and is tied to it.
It only accepts messages from that Converter and sends all rewards to that Converter.
The general flows it provides are:

* Accept Stake message from Converter and execute custom Bond message
* Accept Unstake message from Converter and execute custom Unbond message
* Trigger Reward Withdrawals periodically and send to the Converter

It should keep track on how much it has delegated to each validator, and the total amount of delegations,
along with the max cap. It should not try to bond anything beyond the max cap, as that will error.
But then silently adjust internal bookkeeping, ignoring everything over max cap.

The simplest solution is:

* If total delegated is less than max cap, but new delegation will exceed it, just delegate the difference
* If total delegated is less than max cap and we try to stake more, update accounting, but don't send virtual stake messages
* If we unbond, but total delegated remains greater than max cap, update accounting, but don't send virtual stake messages

However, there is an issue here, as we are staking on different validators. Imagine I have a cap of 100. 80 is bonded to A.
I request to bond 120 more to B, which turns into 20 bond to B. There is an actual ration of 4:1 A:B bonded, but the remote
stakers have delegated at a ration of 2:3. This gets worse if eg. someone unbonds all A. We end up with 120 requested for B,
but 80 delegated to A and 20 delegated to B.

To properly handle this, we should use Redelegations to rebalance when the amounts are updated. This may be quite some messages
(if we have 50 validators, each time we stake one more, we need to add a bit to that one and decrease the other 50... a smart
solution may use some batches). Limiting the interface to custom SDK modules, we would first "immediately unbond"
all other validators, then bond that virtual stake to the newly bonded validator. Much care must be take with rounding issues
to avoid going 0.000001 token above the max cap.

Note: in an alternate implementation this redelegation could be in the native Go module, but we chose to put majority of logic into contracts
for portability.

Note: this example demonstrates why the immediate unbond power is required to properly implement this, and we cannot just do a pure contract
calling normal staking commands (or we would be limited to shifting validators once per unbond period).

### Module

The module maintains a list of addresses (for Virtual Staking contracts), along with a max cap of
virtual tokens for each. It's main purpose is to process `Bond` and `Unbond` messages from
any registered contract up to the max cap. Note that it mints "virtual tokens" that don't affect
max supply queries and can only be used for staking.

The permissions defined in the Meta-Staking module to cap the influence of the various provider is of cirtical importance
for the security design of Mesh Security. Not all remote chains are treated equally and we need to be selective
of how much security we allow to rest on any given token.

The Virtual Staking **Module** maintains an access map `Addr => StakePermission` which can only be updated by chain governance (param change).
The Virtual Staking **Module** also maintains the current state of each of the receivers.

```go
type StakePermission struct {
  /// Limit we cap the virtual stake of this converter.
  /// Defined as a number of "virtual native tokens" this can mint.
  MaxStakingRatio: sdk.Int,
}
```

Beyond MVP, we wish to add the following functionality:

* Provide WithdrawReward callbacks each epoch (eg 1 day) on BeginBlock to all registered contracts
* Provide configuration for optional governance multiplier (eg 1 virtual stake leads to 1 tendermint power,
but may be 0 or 1 or even 0.5 gov voting power)

### Reward Withdrawls

We need to trigger each Virtual Staking contract once an epoch.
This can be done via CronCat or other bot (for MVP), and ideally via a BeginBlock hook for v1.

The Virtual Staking Module also has a BeginBlock hook called once per epoch (param setting, eg 1 day), that will trigger all reward withdrawals.
This epoch has a different start for each converter (based on when they were authorized), so they happen staggered over time.
When the epoch finishes for one Converter, the Module will withdraw rewards from all delegations that converter made, and send those tokens
to the Converter along with the info of which validator these are for.

The initial implementation will call the Converter eg 50 times, once for each validator. If dev time permits, we can use a more optimized
structure and call it once, with all the info it needs to map which token corresponds to which validator. (See both [variants of
`ConverterExecMsg`](#interface))

The Converter in turn will make a number of IBC packets to send the tokens and this metadata back to the External Staking module on the Provider chain.

## Roadmap

Define which pieces are implemented when:

MVP: We stake virtual tokens (like SuperFluid), can unbond rapdily, and these influence governance normally
(validators with delegations get more governance voting power)

V1: We can turn the governance influence on and off per converter. (Stake impacts Tendermint voting power,
but may or may not impact governance voting powerr). We also get native callbacks for triggering rewards
each epoch.

V2: Make improvements here as possible (rebonding, fractional governance multiplier)
