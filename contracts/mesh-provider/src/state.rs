use cosmwasm_schema::cw_serde;

use cosmwasm_std::{Addr, Decimal, Fraction, Uint128};
use cw_controllers::Claims;
use cw_storage_plus::{Item, Map};

use crate::msg::ConsumerInfo;
use crate::ContractError;

#[cw_serde]
pub struct Config {
    pub consumer: ConsumerInfo,
    pub slasher: Option<Addr>,
    /// Address of Lockup contract from which we accept ReceiveClaim
    pub lockup: Addr,
    /// Unbonding period of the remote chain in seconds
    pub unbonding_period: u64,
    /// IBC denom string - "port_id/channel_id/denom"
    pub rewards_ibc_denom: String,
}

pub const CONFIG: Item<Config> = Item::new("config");
pub const PACKET_LIFETIME: Item<u64> = Item::new("packet_time");
pub const CHANNEL: Item<String> = Item::new("channel");
pub const PORT: Item<String> = Item::new("port");

// info on each validator, including voting and slashing
pub const VALIDATORS: Map<&str, Validator> = Map::new("validators");

// map from (delgator, validator) to current stake - stored as shares, previously multiplied
pub const STAKED: Map<(&Addr, &str), Stake> = Map::new("staked");

pub const CLAIMS: Claims = Claims::new("claims");

#[cw_serde]
#[derive(Default)]
pub struct Stake {
    /// how many tokens we have received here
    pub locked: Uint128,
    /// total number of shares bonded
    /// Note: if current value of these shares is less than locked, we have been slashed
    /// and act accordingly
    pub shares: Uint128,
    pub rewards: DelegatorRewards,
}

#[cw_serde]
#[derive(Default)]
pub struct DelegatorRewards {
    pub pending: Decimal,
    pub paid_rewards_per_token: Decimal,
}

impl Stake {
    pub fn new() -> Self {
        Default::default()
    }

    /// How many tokens this is worth at current validator price
    pub fn current_value(&self, val: &Validator) -> Uint128 {
        val.shares_to_tokens(self.shares)
    }

    /// Calculate rewards
    pub fn calc_pending_rewards(
        &mut self,
        new_rewards_per_token: Decimal,
        staked: Uint128,
    ) -> Result<(), ContractError> {
        if staked.is_zero() {
            self.rewards.paid_rewards_per_token = new_rewards_per_token;
            return Ok(());
        }

        let rewards_per_token_to_pay = new_rewards_per_token - self.rewards.paid_rewards_per_token;

        if rewards_per_token_to_pay.is_zero() {
            // Got nothing to calculate, move on
            return Ok(());
        }

        self.rewards.pending += rewards_per_token_to_pay.checked_mul(Decimal::new(staked))?;

        self.rewards.paid_rewards_per_token = new_rewards_per_token;

        Ok(())
    }

    /// Reset pending, keep leftover in pending.
    pub fn reset_pending(&mut self) {
        self.rewards.pending -= self.rewards.pending.floor();
    }

    // TODO: Find a better way of doing this?
    /// Turn pending decimal to u128 to send tokens
    pub fn pending_to_u128(&self) -> Result<u128, ContractError> {
        let full_num = self.rewards.pending.floor().atomics();
        let to_send = full_num.checked_div(Uint128::from(
            10_u32.pow(self.rewards.pending.decimal_places()),
        ))?;
        Ok(to_send.u128())
    }

    /// Check if a slash has occurred. If so, reduced my locked balance and
    /// return the amount that should be slashed. Note: this is mutable and
    /// will return None after the first call.
    pub fn take_slash(&mut self, val: &Validator) -> Option<Uint128> {
        let cur = self.current_value(val);
        if cur == self.locked {
            None
        } else {
            let res = Some(self.locked - cur);
            self.locked = cur;
            res
        }
    }

    /// Add tokens to the validator, update that state as well as our stake
    pub fn stake_validator(&mut self, val: &mut Validator, tokens: impl Into<Uint128>) {
        let tokens = tokens.into();
        let shares = val.stake_tokens(tokens);
        self.locked += tokens;
        self.shares += shares;
    }

    /// Removes stake from the validator
    pub fn unstake_validator(
        &mut self,
        val: &mut Validator,
        tokens: impl Into<Uint128>,
    ) -> Result<(), ContractError> {
        let tokens = tokens.into();
        let shares = val.unstake_tokens(tokens)?;
        self.locked = self
            .locked
            .checked_sub(tokens)
            .map_err(|_| ContractError::InsufficientStake)?;
        self.shares = self
            .shares
            .checked_sub(shares)
            .map_err(|_| ContractError::InsufficientStake)?;
        Ok(())
    }
}

#[cw_serde]
pub struct Validator {
    // how many shares have been staked here
    pub stake: Uint128,
    // multiplier between 1 share and 1 token. Starts at 1, goes down upon slash
    pub multiplier: Decimal,
    // how active is it
    pub status: ValStatus,
    pub rewards: ValidatorRewards,
}

#[cw_serde]
pub struct ValidatorRewards {
    /// rewards_per_token, total of rewards to be paid per staked token
    pub rewards_per_token: Decimal,
}

impl Default for ValidatorRewards {
    fn default() -> Self {
        Self::new()
    }
}

impl ValidatorRewards {
    pub fn new() -> Self {
        ValidatorRewards {
            rewards_per_token: Decimal::zero(),
        }
    }

    pub fn calc_rewards(
        &mut self,
        rewards: Uint128,
        total_tokens: Uint128,
    ) -> Result<(), ContractError> {
        let rewards_dec = Decimal::checked_from_ratio(rewards, total_tokens)?;

        self.rewards_per_token = rewards_dec.checked_add(self.rewards_per_token)?;
        Ok(())
    }
}

#[cw_serde]
pub enum ValStatus {
    Active,
    Removed,
    Tombstoned,
}

impl Default for Validator {
    fn default() -> Self {
        Validator::new()
    }
}

impl Validator {
    pub fn new() -> Self {
        Validator {
            stake: Uint128::zero(),
            multiplier: Decimal::one(),
            status: ValStatus::Active,
            rewards: ValidatorRewards::new(),
        }
    }

    /// Returns value staked in tokens
    pub fn stake_value(&self) -> Uint128 {
        self.shares_to_tokens(self.stake)
    }

    // reduce stake by percentage
    pub fn slash(&mut self, percent: Decimal) {
        let mult = Decimal::one() - percent;
        self.multiplier *= mult;
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
    pub fn unstake_tokens(&mut self, tokens: impl Into<Uint128>) -> Result<Uint128, ContractError> {
        let shares = self.tokens_to_shares(tokens);
        self.stake = self
            .stake
            .checked_sub(shares)
            .map_err(|_| ContractError::InsufficientStake)?;
        Ok(shares)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::Decimal;

    #[test]
    fn validator_stake_unstake() {
        let mut val = Validator::new();
        val.stake_tokens(500u128);
        assert_eq!(val.stake_value().u128(), 500u128);
        val.unstake_tokens(100u128).unwrap();
        assert_eq!(val.stake_value().u128(), 400u128);
        // cannot unstake too much
        let err = val.unstake_tokens(420u128).unwrap_err();
        assert!(matches!(err, ContractError::InsufficientStake));
    }

    #[test]
    fn validator_slashing() {
        let mut val = Validator::new();
        val.stake_tokens(500u128);
        val.slash(Decimal::percent(20));
        assert_eq!(val.stake_value().u128(), 400u128);
        let shares = val.unstake_tokens(200u128).unwrap();
        assert_eq!(shares.u128(), 250u128);
        assert_eq!(val.stake_value().u128(), 200u128);
        val.slash(Decimal::percent(50));
        assert_eq!(val.stake_value().u128(), 100u128);
    }

    #[test]
    fn normal_stake_unstake() {
        let mut val = Validator::new();
        let mut stake = Stake::new();
        stake.stake_validator(&mut val, 500u128);
        let slashed = stake.take_slash(&val);
        assert_eq!(slashed, None);
        stake.unstake_validator(&mut val, 300u128).unwrap();

        assert_eq!(val.stake_value().u128(), 200);
        assert_eq!(stake.current_value(&val).u128(), 200);

        // error on unstaking too much
        stake.unstake_validator(&mut val, 201u128).unwrap_err();
    }

    #[test]
    fn stake_with_slashing() {
        let mut val = Validator::new();
        let mut stake = Stake::new();
        stake.stake_validator(&mut val, 500u128);
        // slash by 20%
        val.slash(Decimal::percent(20));
        // error on trying to unstake too much
        stake.unstake_validator(&mut val, 500u128).unwrap_err();

        // success trying to unstake less
        stake.unstake_validator(&mut val, 300u128).unwrap();

        // now, check the slash is properly calculated
        let slash = stake.take_slash(&val).unwrap();
        assert_eq!(slash.u128(), 100);

        // and only 100 left
        assert_eq!(stake.current_value(&val).u128(), 100);
        // 50 after additional slash by 50%
        val.slash(Decimal::percent(50));
        assert_eq!(stake.current_value(&val).u128(), 50);
    }
}
