use crate::ContractError;
use cosmwasm_schema::cw_serde;
use cosmwasm_std::{ensure, Addr, Decimal, Uint128};
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
    pub pending: Decimal,
    pub paid_rewards_per_token: Decimal,
}

impl ConsumerInfo {
    pub fn new(funds: impl Into<Uint128>) -> Self {
        ConsumerInfo {
            available_funds: funds.into(),
            total_staked: Uint128::zero(),
            rewards: ConsumerRewards {
                pending: Decimal::zero(),
                paid_rewards_per_token: Decimal::zero(),
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
        new_rewards_per_token: Decimal,
        staked: Uint128,
    ) -> Result<(), ContractError> {
        // No stack, so no rewards for him
        if staked.is_zero() {
            self.rewards.paid_rewards_per_token = new_rewards_per_token;
            return Ok(());
        }

        let rewards_per_token_to_pay = new_rewards_per_token - self.rewards.paid_rewards_per_token;

        // We don't need to update anything, nothing to calculate
        if rewards_per_token_to_pay.is_zero() {
            return Ok(());
        }

        self.rewards.pending += rewards_per_token_to_pay.checked_mul(Decimal::new(staked))?;

        self.rewards.paid_rewards_per_token = new_rewards_per_token;

        Ok(())
    }

    pub fn reset_pending_rewards(&mut self) {
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
}

pub const REWARDS_DENOM: Item<String> = Item::new("rewards_denom");

/// Map<(consumer address, validator address), rewards amount>
pub const VALIDATORS_REWARDS: Map<&str, ValidatorRewards> = Map::new("validators_rewards");

#[cw_serde]
pub struct ValidatorRewards {
    /// rewards_per_token, total of rewards to be paid per staked token.
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

/// Map<(consumer address, validator address), Amount>
pub const VALIDATORS_BY_CONSUMER: Map<(&Addr, &str), Uint128> = Map::new("validators_by_consumer");
/// Map<(validator address, consumer address), Amount>
pub const CONSUMERS_BY_VALIDATOR: Map<(&str, &Addr), Uint128> = Map::new("consumers_by_validators");

#[cfg(test)]
mod tests {
    use cosmwasm_std::{Decimal, Uint128};

    use super::{ConsumerInfo, ConsumerRewards, ValidatorRewards};
    use crate::ContractError;

    struct Staked {
        total: Uint128,
        consumer_one: Uint128,
        consumer_two: Uint128,
    }

    impl Staked {
        fn new() -> Self {
            Staked {
                total: Uint128::from(200_u128),
                consumer_one: Uint128::from(100_u128),
                consumer_two: Uint128::from(100_u128),
            }
        }

        fn add_stake_to_consumer_one(&mut self, amount: Uint128) {
            self.total += amount;
            self.consumer_one += amount;
        }

        fn add_stake_to_consumer_two(&mut self, amount: Uint128) {
            self.total += amount;
            self.consumer_two += amount;
        }

        fn reduce_stake_to_consumer_one(&mut self, amount: Uint128) {
            self.total -= amount;
            self.consumer_one -= amount;
        }

        fn reduce_stake_to_consumer_two(&mut self, amount: Uint128) {
            self.total -= amount;
            self.consumer_two -= amount;
        }
    }

    fn add_validator_rewards(
        mut validator_rewards: ValidatorRewards,
        rewards: Uint128,
        total_staked: Uint128,
    ) -> Result<ValidatorRewards, ContractError> {
        validator_rewards.calc_rewards(rewards, total_staked)?;
        Ok(validator_rewards)
    }

    fn calc_consumer_rewards(
        validator_rewards: &ValidatorRewards,
        consumer_rewards: ConsumerRewards,
        staked: Uint128,
    ) -> Result<ConsumerRewards, ContractError> {
        let mut consumer_info = ConsumerInfo::new(Uint128::zero());

        consumer_info.rewards = consumer_rewards;

        consumer_info.calc_pending_rewards(validator_rewards.rewards_per_token, staked)?;
        Ok(consumer_info.rewards)
    }

    #[test]
    fn calculate_rewards() {
        let mut staked = Staked::new();
        let mut validator_rewards = ValidatorRewards::new();
        let mut consumer_one_rewards = ConsumerRewards {
            pending: Decimal::zero(),
            paid_rewards_per_token: Decimal::zero(),
        };

        let mut consumer_two_rewards = ConsumerRewards {
            pending: Decimal::zero(),
            paid_rewards_per_token: Decimal::zero(),
        };

        // add 100 tokens as rewards
        validator_rewards = add_validator_rewards(validator_rewards, Uint128::from(100_u128), staked.total).unwrap();

        // calc consumer rewards and add stake
        consumer_one_rewards = calc_consumer_rewards(&validator_rewards, consumer_one_rewards, staked.consumer_one).unwrap();
        staked.add_stake_to_consumer_one(Uint128::from(100_u128));

        // We have 100 in rewards, so now in pending we should have 50 tokens (50% stake)
        assert_eq!(consumer_one_rewards.paid_rewards_per_token, validator_rewards.rewards_per_token);
        assert_eq!(consumer_one_rewards.pending, Decimal::from_atomics(Uint128::from(50_u128), 18).unwrap());

        // add 300 tokens as rewards
        validator_rewards = add_validator_rewards(validator_rewards, Uint128::from(300_u128), staked.total).unwrap();

        // calc consumer rewards and add stake
        consumer_one_rewards = calc_consumer_rewards(&validator_rewards, consumer_one_rewards, staked.consumer_one).unwrap();
        staked.add_stake_to_consumer_one(Uint128::from(100_u128));

        // We make sure that rewards_per_token is updated
        assert_eq!(consumer_one_rewards.paid_rewards_per_token, validator_rewards.rewards_per_token);
        // We now should have 50 tokens from before + 200 from now.
        assert_eq!(consumer_one_rewards.pending, Decimal::from_atomics(Uint128::from(250_u128), 18).unwrap());

        // Calculate rewards for consumer 2 for the first time.
        consumer_two_rewards = calc_consumer_rewards(&validator_rewards, consumer_two_rewards, staked.consumer_two).unwrap();

        // Consumer 2 should have 50 from first rewards, and 100 from second rewards
        assert_eq!(consumer_two_rewards.pending, Decimal::from_atomics(Uint128::from(150_u128), 18).unwrap());
        // We make sure that the total rewards pending are exactly the rewards we gave (400 u128)
        assert_eq!(consumer_one_rewards.pending.checked_add(consumer_two_rewards.pending).unwrap(), Decimal::from_atomics(Uint128::from(400_u128), 18).unwrap());

    }
}
