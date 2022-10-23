use crate::ContractError;
use cosmwasm_schema::cw_serde;
use cosmwasm_std::{ensure, Addr, Uint128};
use cw_storage_plus::{Item, Map};

#[cw_serde]
pub struct Config {
    pub admin: String,
    pub denom: String,
}

pub const CONFIG: Item<Config> = Item::new("config");

pub const CONSUMERS: Map<&Addr, ConsumerInfo> = Map::new("consumers");

#[cw_serde]
pub struct ConsumerInfo {
    // HACK: Funds available for the contract to stake
    // In the future, this could use a generic Superfluid staking module
    // The amount of these funds will limit the voting power
    pub available_funds: Uint128,
    // Total staked funds, cannot stake more than available funds
    pub total_staked: Uint128,
    pub rewards: ConsumerRewards,
}

#[cw_serde]
pub struct ConsumerRewards {
    pub last_height: u64,
    pub pending: Uint128,
    // Total staked funds, cannot stake more than available funds
    pub last_rptpb: Uint128,
}

impl ConsumerInfo {
    pub fn new(funds: impl Into<Uint128>) -> Self {
        ConsumerInfo {
            available_funds: funds.into(),
            total_staked: Uint128::zero(),
            rewards: ConsumerRewards {
                last_height: 0,
                pending: Uint128::zero(),
                last_rptpb: Uint128::zero(),
            },
        }
    }

    pub fn increase_stake(&mut self, stake: Uint128) -> Result<(), ContractError> {
        let new_stake = self.total_staked + stake;
        ensure!(
            self.available_funds >= new_stake,
            ContractError::NoFundsToDelegate {}
        );
        self.total_staked = new_stake;
        Ok(())
    }

    pub fn decrease_stake(&mut self, stake: Uint128) -> Result<(), ContractError> {
        self.total_staked = self
            .total_staked
            .checked_sub(stake)
            .map_err(|_| ContractError::InsufficientDelegation {})?;
        Ok(())
    }

    pub fn calc_pending_rewards(
        &mut self,
        new_rptpb: Uint128,
        staked: Uint128,
        height: u64,
    ) -> Result<(), ContractError> {
        // If height is 0, its first time we calc (on first delegate)
        // No stack, so no rewards for him
        if staked.is_zero() || self.rewards.last_height == 0 {
            self.rewards.last_rptpb = new_rptpb;
            self.rewards.last_height = height;
            return Ok(());
        }

        let rptpb_to_pay = new_rptpb - self.rewards.last_rptpb;
        let block_to_pay = height - self.rewards.last_height;

        if rptpb_to_pay.is_zero() || block_to_pay == 0 {
            self.rewards.last_rptpb = new_rptpb;
            self.rewards.last_height = height;
            return Err(ContractError::ZeroRewardsToSend {})
        }

        let pending_rewards = rptpb_to_pay
            .checked_mul(staked)?
            .checked_mul(Uint128::from(block_to_pay))?;

        self.rewards.pending += pending_rewards;
        self.rewards.last_height += height;
        self.rewards.last_rptpb += new_rptpb;

        Ok(())
    }
}

pub const REWARDS_DENOM: Item<String> = Item::new("rewards_denom");

/// Map<(consumer address, validator address), rewards amount>
pub const VALIDATORS_REWARDS: Map<&str, ValidatorRewards> = Map::new("validators_rewards");

#[cw_serde]
pub struct ValidatorRewards {
    /// Last height we withdrew rewards and calculated them
    pub last_height: u64,
    /// rewards_per_token_per_block, total of rewards to be paid, per staked token per block.
    pub total_rptpb: Uint128,
}

impl ValidatorRewards {
    pub fn new(height: u64) -> Self {
        ValidatorRewards {
            last_height: height,
            total_rptpb: Uint128::zero(),
        }
    }

    pub fn calc_rewards(
        &mut self,
        rewards: Uint128,
        total_tokens: Uint128,
        curr_height: u64,
    ) -> Result<(), ContractError> {
        let height_diff = curr_height - self.last_height;

        if height_diff == 0 {
            return Err(ContractError::ValidatorRewardsCalculationWrong {});
        }

        self.total_rptpb = rewards
            .checked_div(total_tokens)?
            .checked_div(Uint128::from(height_diff))?
            .checked_add(self.total_rptpb)?;
        Ok(())
    }
}

/// Map<(consumer address, validator address), Amount>
pub const VALIDATORS_BY_CONSUMER: Map<(&Addr, &str), Uint128> = Map::new("validators_by_consumer");
/// Map<(validator address, consumer address), Amount>
pub const CONSUMERS_BY_VALIDATOR: Map<(&str, &Addr), Uint128> = Map::new("consumers_by_validators");
