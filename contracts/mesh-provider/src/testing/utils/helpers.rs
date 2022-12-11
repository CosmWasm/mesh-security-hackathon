use cosmwasm_std::{coins, Addr, Decimal, Uint128};
use cw_multi_test::{App, BankSudo, SudoMsg};

use crate::state::{
    DelegatorRewards, Stake, ValStatus, Validator, ValidatorRewards, STAKED, VALIDATORS,
};

use mesh_testing::{
    constants::{DELEGATOR_ADDR, REWARDS_IBC_DENOM, VALIDATOR},
    macros::addr,
    multitest_helpers::update_storage,
};

/// Function for multi-test to add validator to storage directly and by-pass
/// the IBC call that is needed to do so.
pub fn add_validator(app: &mut App, addr: Addr) {
    update_storage(app, addr.as_bytes(), &mut |storage| {
        VALIDATORS
            .save(storage, VALIDATOR, &Validator::new())
            .unwrap();
    });
}

pub fn add_rewards(app: &mut App, addr: Addr) {
    // Fund the contract to have enough rewards to send
    app.sudo(SudoMsg::Bank(BankSudo::Mint {
        to_address: addr.to_string(),
        amount: coins(100000, REWARDS_IBC_DENOM),
    }))
    .unwrap();

    update_storage(app, addr.as_bytes(), &mut |storage| {
        VALIDATORS
            .save(
                storage,
                VALIDATOR,
                &Validator {
                    stake: Uint128::new(1000),
                    multiplier: Decimal::one(),
                    status: ValStatus::Active,
                    rewards: ValidatorRewards {
                        rewards_per_token: Decimal::from_atomics(1_u128, 0).unwrap(),
                    },
                },
            )
            .unwrap();

        STAKED
            .save(
                storage,
                (&addr!(DELEGATOR_ADDR), VALIDATOR),
                &Stake {
                    locked: Uint128::new(1000),
                    shares: Uint128::new(1000),
                    rewards: DelegatorRewards::default(),
                },
            )
            .unwrap();
    });
}
