# Virtual Staking

Virtual Staking is a permissioned contract on the Consumer side that can interact with the
native staking module in special ways. There are usually multiple Virtual Staking contracts
on one Consumer chain, one for each active Provider, linked 1-to-1 with a Converter.

## Previous Work

There is a lot of overlap with this design and Osmosis' [design of SuperFluid Staking](https://github.com/osmosis-labs/osmosis/tree/main/x/superfluid).
Before commenting on the technical feasibility or implementation on the SDK side, it would be good to review this in depth.
This is probably the biggest change made to staking functionality without forking `x/staking` and should be leveraged for the MVP at least.

(Note to author as well as all reviewers)

## Extensibility

Virtual Staking is primarily the interface exposed to the Convert contract, so we can use wildly
different implementations of it on different chains. Since we want to limit the customizations,
I will describe the simplest workable customizations for a Cosmos SDK chain.

Some chains may want to provide more native powers, and extend the Virtual Staking contract with a custom
one to make use of them. Other architectures, like Picasso or cw-sdk, may have a wildly different architecture
for interacting with the staking module. That would require a Virtual Staking contract that fulfills the
high-level design points below, but not the implementation.

To keep things clear for now, I will focus on the standard contract and module we aim to ship as part
of the basic design.

## Components

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

### Contract

Each Virtual Staking contract is deployed with a Converter contract and is tied to it.
It only accepts messages from that Converter and sends all rewards to that Converter.
The general flows it provides are:

* Accept Stake message from Converter and execute custom Bond message
* Accept Unstake message from Converter and execute custom Unbond message
* Trigger Reward Withdrawals periodically and send to the Converter

### Module

The module maintains a list of addresses (for Virtual Staking contracts), along with a max cap of
virtual tokens for each. It's main purpose is to process `Bond` and `Unbond` messages from
any registered contract up to the max cap. Note that it mints "virtual tokens" that don't affect
max supply and can only be used for staking.

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

The implementation may choose to call the Converter eg 50 times, once for each validator, or call it once, with all the info it needs
to map which token corresponds to which validator. The Converter in turn will make a number of IBC packets to send the tokens and this
metadata back to the External Staking module on the Provider chain.

## Roadmap

Define which pieces are implemented when:

MVP: We stake virtual tokens (like SuperFluid), can unbond rapdily, and these influence governance normally

V1: We can turn the governance influence on and off per converter. We also get native callbacks for
triggering rewards

V2: Make improvements here as possible (rebonding, fractional governance multiplier)
