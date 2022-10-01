# mesh-security

Implementation of Sunny's [Mesh Security](https://youtu.be/Z2ZBKo9-iRs?t=4937) talk from Cosmoverse 20220 (Hackathon / Prototype status)

This should run on any CosmWasm enabled chain. This is MVP design and gives people
hands on use, that should work on a testnet. We will list open questions below that need
to be resolved before we can use this in production.

## Contracts

* `meta-staking` - a bridge between the rest of the contracts and the x/staking module to
  provide a consistent, friendly interface for our use case
* `ilp` - an "Illiquidity Pool" contract that locks tokens and issues multiple claims
  to other consumers, who can all slash that stake and eventually release their claim
* `mesh-provider` - IBC-enabled contract that issues claims on an ILP and speaks IBC to a consumer
* `mesh-consumer` - an IBC-enabled contract that receives messages from `ibc-provider` and
  communicates with `meta-staking` to update the local delegations / validator power

## Overview for Users

**High Level:** You connect Osmosis to Juno and Juno to Osmosis. We will only look at one side
of this, but each chain is able to be both a consumer and producer at the same time.
You can also connect each chain as a provider to N chains and a consumer from N chains.

Let's analyze the Osmosis side of this. Osmosis is the provider of security to Juno.
Once the contracts have been deployed, a user can interact with this as follows.

Cross-staking:

1. User stakes their tokens in ILP contract on Osmosis
2. User can cross-stake those tokens to a Juno provider contract (on Osmosis), specifying how many of their 
   tokens to cross-stake and to which validator
3. The Osmosis consumer contract (on Juno) receives a message from the counterparty and updates
   the stake in the `meta-staking` contract.
4. The `meta-staking` contract checks the values and updates it's delegations to `x/staking` accordingly.

Claiming Rewards:

1. Anyone can trigger the Osmosis consumer contract to claim rewards from the `meta-staking` contract
2. The Osmosis consumer contract sends tokens to the Juno provider contract (via ics20)
3. The Osmosis consumer contract sends a message to the Juno provider contract to inform
   of the new distribution (and how many go to which validator).
4. The Juno provider contract updates distribution info to all stakers, allowing them to claim
   their share of the $JUNO rewards on Osmosis.

Unstaking:

1. A user unstakes their tokens from the Juno provider contract (on Osmosis)
2. We update the local distribution info to reflect the new amount of tokens staked
3. This sends a message to the Osmosis consumer contract, which updates the `meta-staking` contract
   to remove the delegation.
4. After the unbonding period has passed (eg. 2 weeks, 4 weeks) the `meta-staking` contract
   informs the Osmosis consumer contract that the tokens are available to be withdrawn.
5. The Osmosis consumer contract sends a message to the Juno provider contract that the stake is unbonded.
6. The Juno provider contract sends a message to the Osmosis ILP contract to release the claims.

Slashing:

1. A slashing event occurs for a validator on the consumer chain (Juno)
2. Someone calls a method to submit evidence on the Osmosis `mesh-consumer` contract
3. The `mesh-consumer` contract veries that a slashing event has indeed occurred on the Juno chain and fires off 
   an IBC packet to the `mesh-provider` contract on the Osmosis chain containing information about the slashing
   event.
4. The `mesh-provider` updates the claims of everyone delegating to the offending validator. Tokens are unbonded
   and scheduled to be burned.
5. `mesh-provider` sends IBC packet updates to other chains about the new voting power.

Claiming ILP tokens:

A user can stake any number of tokens to the ILP, and use them in multiple provider contracts.
The ILP ensures that the user has balance greater than or equal to max(claims) at all time.
If you put in eg 1000 OSMO, but then provide 700, 500, and 300 to various providers,
you can pull out 300 OSMO from the ILP. Once you successfully release the claim on the
provider with 700, then you can pull out another 200 OSMO.

## Overview for Installing

1. Deploy the contracts to Osmosis and Juno
2. `x/gov` on Juno will tell the "Osmosis consumer contract" which `(connectionId, portId)` to trust
3. `x/gov` will provide the "Osmosis consumer contract" with some JUNO tokens with which it can later delegate
   (This is a known hack... we discuss improvement below).
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
    that work like normal, but are removed from the "totalSupply" query via offet.
  * Fork `x/staking` to allow such synthetic delegations that don't need tokens.
    This is a hard lift, but would allow custom logic, like counting those tokens
    in tendermint voting power, but exclude them from `x/gov`, and decide on some
    reducing factor for their rewards.
* How to cleanly define limits for the providing chains on how much power they can
  have on the consuming chain? We start with a fixed number (# of JUNO), but better
  to do something like "max 10% of total staked tokens".
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
  rewards in another way, especially when we modify `x/staking`.