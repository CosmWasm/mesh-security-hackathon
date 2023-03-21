# Stake Receiver

The Stake Receiver is on the consumer side and connected an External Staker on the Provider side.
This handles the normalization of the token streams and communicates with "Virtual Staking".
It is connected to the Provider chain via IBC and "receives" the various staking commands

**TODO** This was called the "Consumer contract" in previous versions and probably needs a better name

## Setup

When we [deploy the contracts](../ibc/Overview.md#deployment), we connect the Stake Receiver on the consumer chain
with an [External Staking](../provider/ExternalStaking.md) contract on the Provider. Once this connection is established,
Consumer governance can authorize this Stake Receiver with some ability to mint on the ["Virtual Staking" contract](./VirtualStaking.md).

Note that when we deploy the Stake Receiver, we configure the address of the "virtual staking" contract. We must also
define a price oracle contract on setup. (see ["Price Normalization"](#price-normalization) below)

## Staking Flow

Once the connection is established, the provider can send various "virtual stake" messages to the receiver, which is responsible
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

Thus this cross-stake will trigger the receiver to request the virtual staking module to stake 1080 CONS.

### Virtual Staking

**TODO** Call the "virtual staking" module

(Note, we must set the validators. Where do we maintain state? Just diffs between recevier and "virtual staking"?)

## Rewards Flow

**TODO**

## Unstaking Flow

**TODO**

