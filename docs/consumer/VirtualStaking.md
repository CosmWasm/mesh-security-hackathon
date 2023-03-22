# Virtual Staking

Virtual Staking is a permissioned contract on the Consumer side that can interact with the
native staking module in special ways. It manages an allow list of authorized Receivers
and is responsible for converting their "virtual stake" into actual stake, as well
as providing them with their share of the rewards.

## Components

Virtual Staking will be a mix of a Cosmos SDK module and a CosmWasm contract. The contract will provide
the single point of access to the module and be the only address that can call it. The SDK module will
manage the actual interactions with the native staking module. I will call them "Virtual Staking Contract"
and "Virtual Staking Module" (or just "Contract" and "Module") in this document. Outside of this document,
we will usually refer to them together as one single unit, "Virtual Staking".

### Contract

The contract must provide the following:

* Interface for the ["Stake Receiver"](./Receiver.md) to "virtually stake"
* An list of Receivers and their allowed maximums set by on-chain governance
* A query interface to the above
* Ability to send staking reward tokens to the receiver contracts

### Module

The module must provide the following:

* Config address for registered virtual staking contract to "virtually stake"
* Mints "virtual tokens" (that don't affect supply) and stakes them to validators (like Osmosis' superfluid staking module)
* Handle unbondings cheaply (ideally not the "7 pending" limit)
* V1/V2: Configuration whether "virtual tokens" also count in governance voting.

## Functionality

The Virtual Staking **Module** maintains an access map `Addr => StakePermission` which can only be updated by chain governance (param change).
The Virtual Staking **Module** also maintains the current state of each of the receivers.

```go
type StakePermission struct {
  /// Limit we cap the virtual stake of this receiver.
  /// Defined as a multiplier to the total amount of native stake.
  /// eg. 1.0 means this receiver can "virtually stake" as much as the native module, giving 50-50 split if there is only one receiver
  MaxStakingRatio: sdk.Dec,
  
  /// Virtual stake always contributes to the tendermint voting power.
  /// This defines much the virtual stake contributes to the x/gov voting power.
  /// It must range between 0.0 (stake doesn't give validator any more gov power)
  /// to 1.0 (stake gives same gov power as normal native staking).
  /// 
  /// In MVP, the only allowed setting is 1.0 (treat it like normal staking)
  /// In 1.0, we improve the native module, allowing minimally 0.0 and 1.0 (boolean flag)
  /// By 2.0, if possible, we will allow all values between 0.0 and 1.0
  GovMultiplier: sdk.Dec,
}
```

The Virtual Staking Module also has a BeginBlock hook called once per epoch (param setting, eg 1 day), that will trigger all reward withdrawals.
This epoch has a different start for each receiver (based on when they were authorized), so they happen staggered over time.
When the epoch finishes for one Receiver, the Module will withdraw rewards from all delegations that receiver made, and send those tokens
to the Receiver along with the info of which validator these are for.

The implementation may choose to call the Receiver eg 50 times, once for each validator, or call it once, with all the info it needs
to map which token corresponds to which validator. The Receiver in turn will make a number of IBC packets to send the tokens and this
metadata back to the External Staking module on the Provider chain.

**TODO** Question: as I design this, I realize the params and all access control are best inside the module. So it can eg. update the current stake
when the native staking supply changes (using sdk hooks). The contract is becoming a very light-weight wrapper, and maybe best just to add some CustomMsg
here. Receivers directly call the Virtual Staking Module and there is no need to have a contract. **I would like some feedback on this change**

## Roadmap

Define which pieces are implemented when:

MVP: We stake virtual tokens (like SuperFluid), but have no special unbonding power, and these influence governance normally

V1: We can turn the governance influence on and off per receiver. We can also unbond virtual stake immediately as

V2: Make improvements here as possible (rebonding, fractional governance multiplier)
