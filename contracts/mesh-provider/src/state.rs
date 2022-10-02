use cosmwasm_std::{Addr, Decimal, Fraction, Uint128};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cw_storage_plus::{Item, Map};

use crate::msg::ConsumerInfo;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    pub consumer: ConsumerInfo,
    pub slasher: Option<Addr>,
    /// Address of Lockup contract from which we accept ReceiveClaim
    pub lockup: Addr,
    /// Unbonding period of the remote chain in seconds
    pub unbonding_period: u64,
}

pub const CONFIG: Item<Config> = Item::new("config");
pub const CHANNEL: Item<String> = Item::new("channel");

// info on each validator, including voting and slashing
pub const VALIDATORS: Map<&str, Validator> = Map::new("validators");

// map from (delgator, validator) to current stake - stored as shares, previously multiplied
pub const STAKED: Map<(&Addr, &str), Uint128> = Map::new("staked");

// TODO: Claims
// TODO: rewards

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Validator {
    // how many shares have been staked here
    pub stake: Uint128,
    // multiplier between 1 share and 1 token. Starts at 1, goes down upon slash
    pub multiplier: Decimal,
    // how active is it
    pub status: ValStatus,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub enum ValStatus {
    Active,
    Removed,
    Tombstoned,
}

impl Validator {
    pub fn new() -> Self {
        Validator {
            stake: Uint128::zero(),
            multiplier: Decimal::one(),
            status: ValStatus::Active,
        }
    }

    /// Returns value staked in tokens
    pub fn stake_value(&self) -> Uint128 {
        self.shares_to_tokens(self.stake)
    }

    // reduce stake by percentage
    pub fn slash(&mut self, percent: Decimal) {
        let mult = Decimal::one() - percent;
        self.multiplier = self.multiplier * mult;
    }

    /// Returns value staked in tokens
    #[inline]
    pub fn shares_to_tokens(&self, shares: impl Into<Uint128>) -> Uint128 {
        self.multiplier * shares.into()
    }

    #[inline]
    pub fn tokens_to_shares(&self, tokens: impl Into<Uint128>) -> Uint128 {
        // TODO: use Uint256 to avoid overflow
        tokens.into() * self.multiplier.denominator() / self.multiplier.numerator()
    }

    /// Increments the local stake and returns the number of shares
    pub fn stake_tokens(&mut self, tokens: impl Into<Uint128>) -> Uint128 {
        let shares = self.tokens_to_shares(tokens);
        self.stake += shares;
        shares
    }

    /// Reduces the local stake and returns the number of shares
    pub fn unstake_tokens(&mut self, tokens: impl Into<Uint128>) -> Uint128 {
        let shares = self.tokens_to_shares(tokens);
        self.stake -= shares;
        shares
    }
}

#[cfg(test)]
mod tests {
    use crate::state::Validator;
    use cosmwasm_std::Decimal;

    #[test]
    fn validator_stake_unstake() {
        let mut val = Validator::new();
        val.stake_tokens(500u128);
        assert_eq!(val.stake_value().u128(), 500u128);
        val.unstake_tokens(100u128);
        assert_eq!(val.stake_value().u128(), 400u128);
    }

    #[test]
    fn validator_slashing() {
        let mut val = Validator::new();
        val.stake_tokens(500u128);
        val.slash(Decimal::percent(20));
        assert_eq!(val.stake_value().u128(), 400u128);
        let shares = val.unstake_tokens(200u128);
        assert_eq!(shares.u128(), 250u128);
        assert_eq!(val.stake_value().u128(), 200u128);
        val.slash(Decimal::percent(50));
        assert_eq!(val.stake_value().u128(), 100u128);
    }
}
