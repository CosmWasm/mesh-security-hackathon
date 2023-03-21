# Vault

The entry point of Mesh Security is the **Vault**. This is where a potential
staker can provide collateral in the form of native tokens, with which he or she wants
to stake on multiple chains.

Connected to the _Vault_ contract, is exactly one [local Staking contract](./LocalStaking.md)
which can delegate the actual token to the native staking module. It also can connect to an
arbitrary number of [external staking contracts](./ExternalStaking.md) which can make use
of said collateral as "virtual stake" to use in an external staking system (that doesn't
use the vault token as collateral). 

The key here is that we can safely use the
same collateral for multiple protocols, as the maximum slashing penalty is significantly
below 100%. If double-signing is penalized by a 5% slash (typical in Cosmos SDK chains),
then one could safely use the same collateral to provide security to 20 chains, as even
if every validator that used that collateral double-signed, there would still be enough
stake to slash to cover that security promise.

As discussed in the higher-level description of the provider design, about extending
this [concept to local DAOs](./Provider.md#dao-dao-extension),
there may be many different implementations of both the _Local Staking_ concept as well
as the _External Staking_ concept. However, we must define
standard interfaces here that can plug into the Vault.

We define this interface as a _Creditor_ (as it accepts leins).

## Definitions

TODO: refine this

* **Native Token** - The native staking token of this blockchain. More specifically,
  the token in which all collateral is measured.
* **Slashable Collateral** - `Leins(user).map(|x| x.amount * x.slashable).sum()`
* **Maximum Lein** - `Leins(user).map(|x| x.amount).max()`
* **Free Collateral** - `Collateral(user) - max(SlashableCollateral(user), MaximumLein(user))`

## Design Decisions

The _vault_ contract requires one canonical _Local Staking_ contract to be defined when it is
created, and this contract address cannot be changed.

The _vault_ contract doesn't require the _External Stakers_ to be pre-registered. Each user can decide
which external staker it trusts with their tokens. (We will provide guidance in the UI to only
show "recommended" externals, but do not enforce on the contract level, if someone wants to build their own UI)

The _vault_ contract enforces the maximum amount a given Creditor can slash to whatever was
agreed when making the lien.

The _vault_ contract will only release a lien when requested by the creditor. (No auto-release override).

The _vault_ contract may force-reduce the size of the lien only in the case of slashing by another creditor.
The creditors must be able to handle this case properly.

The _vault_ contract ensures the user's collateral is sufficient to service all the liens
made on said collateral.

The _vault_ contract may have a parameter to limit slashable collateral or maximum lien to less than
100% of the size of the collateral. This makes handling a small slash condition much simpler. 

The _creditor_ is informed of a new lien and may reject it by returning an error
(eg if the slashing percentage is too small, or if the total credit would be too high).

The _creditor_ may slash the collateral up to the agreed upon amount when it was lent out.

The _creditor_ should release the lien once the user terminates any agreement with the creditor.

## Implementation

**TODO** translate the below into Rust code. After writing this in text, I realize
it is much less clear than the corresponding code.

### State

* Collateral: `User -> Amount`
* Leins: `User -> {Creditor, Amount, Slashable}[]`
* Credit `Creditor -> Amount`

### Invariants

* `SlashableCollateral(user) <= Collateral(user)` - for all users
* `MaximumLein(user) <= Collateral(user)` - for all users
* `Leins(user).map(|x| x.creditor).isUnique()` - for all users

### Transitions

**Provide Collateral**

Any user may deposit native tokens to the vault contract,
thus increasing their collateral as stored in this contract.

**Withdraw Collateral**

Any user may withdraw any _Free Collateral_ credited to their account.
Their collateral is reduced by this amount and these native tokens are
immediately transferred to their account.

**Provide Lein**

TODO. Promise collateral as slashable to some creditor.
Args `(creditor, amount, slashable)`. 
This is updated locally

**Release Lein**

TODO

**Slash**

TODO

* Increase Slashing(user, creditor)?

## Footnotes

For MVP, Slashable Collateral and Maximum Lein can be up to 100% of total Collateral.
