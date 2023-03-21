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

**TODO** Explain how this works from a high-level perspective, including the native staking module

## Implementation

**TODO** Define which pieces are implemented where (also think of MVP, v1, v2)

Who triggers reward withdrawal? BeginBlockers in module? Or CronCat?
