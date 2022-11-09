use std::ops::Sub;

use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Decimal, Fraction, Uint128};
use cw_storage_plus::{Item, Map};

use crate::{msg::ProviderInfo, ContractError};

#[cw_serde]
pub struct Config {
    pub provider: ProviderInfo,
    pub remote_to_local_exchange_rate: Decimal,
    pub meta_staking_contract_address: Addr,
    pub ics20_channel: String,
}

pub const CONFIG: Item<Config> = Item::new("config");
pub const PACKET_LIFETIME: Item<u64> = Item::new("packet_time");
pub const CHANNEL: Item<String> = Item::new("channel");

// info on each validator, including voting and slashing
pub const VALIDATORS: Map<&str, Validator> = Map::new("validators");

// map from (delgator, validator) to current stake - stored as shares, previously multiplied
pub const STAKED: Map<(&str, &str), Stake> = Map::new("staked");

#[cw_serde]
pub struct Stake {
    /// how many tokens we have received here
    pub locked: Uint128,
    /// total number of shares bonded
    /// Note: if current value of these shares is less than locked, we have been slashed
    /// and act accordingly
    pub shares: Uint128,
    pub rewards: DelegatorRewards,
}

impl Default for Stake {
    fn default() -> Self {
        Self::new()
    }
}

#[cw_serde]
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
        let shares = val.unstake_tokens(tokens);
        // math checks should already have happened on provider.
        self.locked = self.locked.sub(tokens);
        self.shares = self.shares.sub(shares);
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
    pub rewards_per_token: Decimal,
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
            rewards_per_token: Decimal::zero(),
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
    pub fn unstake_tokens(&mut self, tokens: impl Into<Uint128>) -> Uint128 {
        let shares = self.tokens_to_shares(tokens);
        // We already did this in provider, so the data we get should already pass the checked_sub method.
        self.stake = self.stake.sub(shares);
        shares
    }

    /// Calculate the new total rewards per token.
    pub fn calc_rewards(&mut self, rewards: Uint128) -> Result<(), ContractError> {
        let total_tokens = self.shares_to_tokens(self.stake);

        // if we have no stake, we probably also don't have rewards (because we check for rewards everytime stake changes)
        // so just return ok.
        if total_tokens.is_zero() {
            return Ok(());
        }

        let rewards_dec = Decimal::checked_from_ratio(rewards, total_tokens)?;

        self.rewards_per_token = rewards_dec.checked_add(self.rewards_per_token)?;
        Ok(())
    }
}
