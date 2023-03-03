# Vault

The entry point of Mesh Security is the **Vault**. This is where a potential
staker can provide collateral in the form of native tokens, with which he or she wants
to stake on multiple chains. 

Connected to the _Vault_ contract, are multiple **Creditor** contracts which can make use
of said collateral to use in a staking system. The key here is that we can safely use the
same collateral for multiple protocols, as the maximum slashing penalty is significantly
below 100%. If double-signing is penalized by a 5% slash (typical in Cosmos SDK chains),
then one could safely use the same collateral to provide security to 20 chains, as even
if every validator that used that collateral double-signed, there would still be enough
stake to slash to cover that security promise.

There may be many different implementations of the _Creditor_ interface, but we define a
standard interface here that is required to work well with the _Vault_.

## Definitions

TODO: refine this

* **Native Token** - The native staking token of this blockchain. More specifically,
  the token in which all collateral is measured.
* **Slashable Collateral** - `Leins(user).map(|x| x.amount * x.slashable).sum()`
* **Maximum Lein** - `Leins(user).map(|x| x.amount).max()`
* **Free Collateral** - `Collateral(user) - max(SlashableCollateral(user), MaximumLein(user))`

## Design Decisions

The _vault_ contract doesn't require the creditors to be pre-registered. Each user can decide
this creditor it trusts with their tokens.

The _vault_ contract enforces the maximum amount a given creditor can slash to whatever was
agreed when making the lean.

The _vault_ contract will only release a lein when requested by the creditor. (No auto-release override).

The _vault_ contract may force-reduce the size of the lein only in the case of slashing by another creditor.
The creditors must be able to handle this case properly.

The _vault_ contract ensures the user's collateral is sufficient to service all the leins
made on said collateral.

The _vault_ contract may have a parameter to limit slashable collateral or maximum lein to less than
100% of the size of the collateral. This makes handling a small slash condition much simpler. 

The _creditor_ is informed of a new lein and may reject it by erroring (eg if the slashing percentage
is too small, or if the total credit would be too high.

The _creditor_ may slash the collateral up to the agreed upon amount when it was lent out.

The _creditor_ should release the lein once the user terminates any agreement with the creditor.

## State

* Collateral: `User -> Amount`
* Leins: `User -> {Creditor, Amount, Slashable}[]`
* Credit `Creditor -> Amount`

## Invariants

* `SlashableCollateral(user) <= Collateral(user)` - for all users
* `MaximumLein(user) <= Collateral(user)` - for all users
* `Leins(user).map(|x| x.creditor).isUnique()` - for all users

## Transitions

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

## Diagrams

TODO

## Footnotes

For MVP, Slashable Collateral and Maximum Lein can be up to 100% of total Collateral.
