# Stake Converter

The Stake Converter is on the consumer side and is connected to an External Staker on the Provider side.
This handles the normalization of the external tokens and _converts_ them into "Virtual Stake".
There is a 1:1 connection between a Converter and a [Virtual Staking Contract](./VirtualStaking.md)
which handles the actual issuance.
The converter is connected to the Provider chain via IBC and handles the various packets coming from it.

**TODO** This was called the "Consumer contract" in previous versions and probably needs a better name

## Setup

When we [deploy the contracts](../ibc/Overview.md#deployment), we connect the Stake Converter on the consumer chain
with an [External Staking](../provider/ExternalStaking.md) contract on the Provider. Once this connection is established,
Consumer governance can authorize this Stake Converter with some ability to mint on the ["Virtual Staking" contract](./VirtualStaking.md).

Note that when we deploy the Stake Converter, we configure the address of the "virtual staking" contract. We must also
define a price oracle contract on setup. (see ["Price Normalization"](#price-normalization) below)

## Staking Flow

Once the connection is established, the provider can send various "virtual stake" messages to the converter, which is responsible
for processing them and normalizing for the local "virtual staking" module.

### Price normalization

When we receive a "virtual stake" message for 1 provider token, we need to perform a few steps to normalize it to the
local staking tokens. 

The first step is simply doing a price conversion. The actual logic giving the price feed is located in an Oracle contract (configured upon init).
We recommend using an (eg daily) TWAP on a DEX with good liquidity - ideally on the consumer chain, but this implementation is left up
to the particular chain it is being deployed on. With this TWAP we convert eg. 1 PROV to 18 CONS.

The second step is to apply a discount. This discount reduced the value of the cross-stake to a value below what we would get from the pure
currency conversion above. This has two purposes: the first is to provide a margin of error when the price deviates far from the TWAP, so
the cross-stake is not overvalued above native staking; the second is to encourage local staking over remote staking. Loooking at the asset's historical
volatility can provide a good estimate for the first step, as a floor for minimum discount. Beyond that, consumer chain tokenomics and governance
design is free to increase the discount as they feel beneficial.

In this case, let's assume a discount of 40%. A user on the provider chain cross-stakes 100 PROV. We end up with a weight of

`100 PROV * 18 CONS/PROV * (1 - 0.4) = 1080 CONS`

Thus this cross-stake will trigger the converter to request the virtual staking module to stake 1080 CONS.

### Virtual Staking

(Note: Current design of [Virtual Staking](./VirtualStaking) is based around `CustomMsg` rather than a contract)

Each converter can call into the Virtual Staking Module, declaring it delegates N `CONS` tokens to validator `V`.
It may delegate many times to the same or different validators. The Virtual Staking Module ensures that the Converter address
is authorized and applies any caps to the impact of the virtual stake.

The VirtualStaking module makes all delegations in the name of the Converter contract, which allows the native x/staking
book keeping to properly split the rewards among each Converter.

## Rewards Flow

Once per epoch, the virtual staking module will trigger rewards. This will send a number of messages to the Converter,
specifying which validators the rewards belong to, along with the native reward tokens themselves. 

The Converter will then [transfer all these tokens via ICS20](../ibc/Overview.md) to the corresponding `External Staking` contract
on the Provider chain, and send a message over the standard IBC channel to inform the `External Staking` contract how to distribute them.
(If we get callbacks on ics20, we send the metadata only after tokens have arrived. Until then (for MVP), we send them concurrently and hope)

## Unstaking Flow

The Converter can also unstake some tokens. These will be held in escrow on the Provider and are susceptible to slashing upon proper evidence
submission. Since the virtual stake is, well, "virtual" and slashing has no impact, the delegation numbers can be immediately reduced
on the consumer's native staking module.

For MVP, we just trigger and unbonding, and when the tokens return to the Virtual Staking Module, they can be burnt (or reused for future delegations).
The native x/staking module limits us to 7 simultaneous unbonding (per Converter), so we need to queue these up and execute them in batches.
This is a standard limitation of liquid staking modules.  For more explanation, [see the stride docs](https://docs.stride.zone/docs/unstaking):

> The Stride blockchain initiates the unbonding process by grouping the records of all of the unbondings on the chain.
> Unbondings are grouped because Cosmos chains do not allow more than 7 unbondings at a time within a 21 day period.
> This is a security measure put in place across the Cosmos ecosystem. This does not impact the average user, but it 
> is the reason Stride processes requests every 4 days.

For V1, we modify the staking module to treat virtual stake specially and can just directly update the stake, without adding to the unbonding queue.
This will allow us to perform more than 7 unbonding simulataneously.

