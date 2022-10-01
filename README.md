# mesh-security

Implementation of Sunny's [Mesh Security](https://youtu.be/Z2ZBKo9-iRs?t=4937) talk from Cosmoverse 2022 (Hackathon / Prototype status)

This should run on any CosmWasm enabled chain. This is MVP design and gives people
hands on use, that should work on a testnet. We will list open questions below that need
to be resolved before we can use this in production.

## Contracts

* `meta-staking` - a bridge between the rest of the contracts and the x/staking module to
  provide a consistent, friendly interface for our use case
* `ilp` - an "Illiquidity Pool" contract that locks tokens and allows lockers to issue multiple claims
  to other consumers, who can all slash that stake and eventually release their claim
* `mesh-provider` - an IBC-enabled contract that issues claims on an ILP and speaks IBC to a consumer. It
  is responsible for submitting slashes it receives from the `slasher` to the `ilp` contract.
* `mesh-consumer` - an IBC-enabled contract that receives messages from `ibc-provider` and
  communicates with `meta-staking` to update the local delegations / validator power
* `slasher` - a contract that is authorized by the `mesh-provider` to submit slashes to it. There can
  be many types of slasher contracts (for different types of evidenses of misbehaviors)

## Overview for Users

**High Level:** You connect Osmosis to Juno and Juno to Osmosis. We will only look at one side
of this, but each chain is able to be both a consumer and producer at the same time.
You can also connect each chain as a provider to N chains and a consumer from N chains.

Let's analyze the Osmosis side of this. Osmosis is the provider of security to Juno.
Once the contracts have been deployed, a user can interact with this as follows.

Cross-staking:

1. User stakes their tokens in ILP contract on Osmosis
2. User can cross-stake those tokens to a Juno `mesh-provider` contract (on Osmosis), specifying how many of their 
   tokens to cross-stake and to which validator
3. The Osmosis `mesh-consumer` contract (on Juno) receives a message from the counterparty `mesh-provider` contract
   and updates the stake in the `meta-staking` contract (on Juno).
4. The `meta-staking` contract checks the values and updates it's delegations to `x/staking` accordingly. (The
   meta-staking contract is assumed to have enough JUNO tokens to do the delegations. How it gets that JUNO is
   out of scope.)

Claiming Rewards:

1. Anyone can trigger the Osmosis consumer contract to claim rewards from the `meta-staking` contract
2. The `mesh-consumer` contract (on JUNO) sends tokens to the `mesh-provider` contract (on Osmosis) via ics20
3. The `mesh-consumer` contract sends a message to the `mesh-provider` contract to inform
   of the new distribution (and how many go to which validator).
4. The `mesh-provider` (on Osmosis) contract updates distribution info to all stakers, allowing them to claim
   their share of the $JUNO rewards on Osmosis.

Unstaking:

1. A user submits a request to unstake their tokens from the `mesh-provider` contract (on Osmosis)
2. We update the local distribution info to reflect the new amount of tokens staked
3. This sends a message to the `mesh-consumer` contract (on Juno), which updates the Juno `meta-staking` contract
   to remove the delegation.
4. The `mesh-provider` contract (on Osmosis) gets the unbonding period for this cross stake by querying
   the `slasher` contract
5. After the unbonding period has passed (eg. 2 weeks, 4 weeks) the `mesh-provider` contract
   informs the `ilp` contract that it removes its claim.
6. If the user's stake in the `ilp` contract has not more claims on it, they can withdraw their stake.

Slashing:

1. Someone calls a method to submit evidence of Juno misbehavior on the `slasher` contract (on Osmosis).
2. The `slasher` contract verifies that a slashing event has indeed occured and makes a contract call to the
   `mesh-provider` contract with the amount to slash.
3. The `mesh-provider` updates the `ilp` stakes of everyone delegating to the offending validator. Tokens are unbonded
   and scheduled to be burned.
4. `mesh-provider` sends IBC packet updates to the `mesh-consumer`s on all other chains about the new voting power.

Claiming ILP tokens:

A user can stake any number of tokens to the ILP, and use them in multiple provider contracts.
The ILP ensures that the user has balance >= the max claim at all times.
If you put in eg 1000 OSMO, but then provide 700, 500, and 300 to various providers,
you can pull out 300 OSMO from the ILP. Once you successfully release the claim on the
provider with 700, then you can pull out another 200 OSMO.

## Overview for Installing

1. Deploy the contracts to Osmosis and Juno
2. `x/gov` on Juno will tell the "Osmosis consumer contract" which `(connectionId, portId)` to trust
3. `x/gov` will provide the "Osmosis consumer contract" with some JUNO tokens with which it can later delegate
   (This is a hacky solution to give the consumer contract OSMO to be able to stake... we discuss improvement below).
4. A relayer connects the two contracts, the consumer contract ensures that the channel is made
   from the authorized `(connectionId, portId)` or rejects it in the channel handshake. It also
   ensures only one channel exists at a time.
5. Once the trusted connection is established and the consumer contract has been granted sufficient
   delegation power, then the user flow above can be used.

## Features to add

These are well-defined but removed from the MVP for simplicity. We can add them later.

* ILP must also allow local staking, and tie into the meta-staking contract to use that
  same stake to provide security on the home chain.

## Open Questions

These are unclear and need to be discussed and resolved further.

* How to cleanly grant consumer contracts the proper delegation power?
  * Something like "superfluid staking" module where we can mint "synthetic staking tokens"
    that work like normal, but are removed from the "totalSupply" query via offset.
  * Fork `x/staking` to allow such synthetic delegations that don't need tokens.
    This is a hard lift, but would allow custom logic, like counting those tokens
    in tendermint voting power, but exclude them from `x/gov`, and decide on some
    reducing factor for their rewards.
* How to cleanly define limits for the providing chains on how much power they can
  have on the consuming chain? We start with a fixed number (# of JUNO), but better
  to do something like "max 10% of total staking power".
* Ensure a minimum voting power for the local stake. If we let 3 chains each use up to 30%
  of the voting power, and they all stake to the max, then we only have 10% of the power locally.
  We can set a minimum to say 40% local, and if all remote chains stake to the max, their
  relative powers are reduced proportionally to ensure this local minimum stake.
* How to normalize the token values? If we stake 2 million $OSMO, we need to convert that
  to the same $$ value of $JUNO before using it to calculate staking power on the Juno chain.
* How to properly handle slashing, especially how a slashing on JUNO triggers a slash on OSMO,
  which should then reduce the voting power of the correlated validators on STARS
  (that was based on the same OSMO stake). This is a bit tricky... TODO: Jake
* Desired reward payout mechanism. For MVP, we treat this as a normal delegator and
  send the tokens back to the provider chain to be distributed. But maybe we calculate
  rewards in another way, especially when we modify `x/staking`. Should also be computationally
  efficient
