# mesh-security

Implementation of Sunny's [Mesh Security](https://youtu.be/Z2ZBKo9-iRs?t=4937) talk from Cosmoverse 20220 (Hackathon / Prototype status)

This should run on any CosmWasm enabled chain. This is MVP design and gives people
hands on use, that should work on a testnet. We will list open questions below that need
to be resolved before we can use this in production.

## Contracts

* `meta-staking` - a bridge between the rest of the contracts and the x/staking module to
  provide a consistent, friendly interface for our use case
* `ilp` - an "IlLiquidity Pool" contract that locks tokens and issues multiple claims
  to other consumers, who can all slash that stake and eventually release their claim
* `mesh-provider` - IBC-enabled contract that issues claims on an ILP and speaks IBC to a consumer
* `mesh-consumer` - an IBC-enabled contract that receives messages from `ibc-provider` and
  communicates with `meta-staking` to update the local delegations / validator power

## Overview

High Level: You connect Osmosis to Juno and Juno to Osmosis. We will only look at one side
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

Claiming ILP tokens:

A user can stake any number of tokens to the ILP, and use them in multiple provider contracts.
The ILP ensures that the user has balance greater than or equal to max(claims) at all time.
If you put in eg 1000 OSMO, but then provide 700, 500, and 300 to various providers,
you can pull out 300 OSMO from the ILP. Once you successfully release the claim on the
provider with 700, then you can pull out another 200 OSMO.




## Open Questions