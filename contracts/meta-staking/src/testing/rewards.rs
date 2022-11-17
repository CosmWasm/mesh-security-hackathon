use std::ops::Deref;

use cosmwasm_std::Uint128;
use mesh_testing::app_wrapper::{self, AppExecute, AppQuery, AppWrapper};

use crate::{
    error::ContractError,
    msg::{ExecuteMsg, QueryMsg},
    state::{ConsumerInfo, CONSUMERS},
    testing::utils::{rewards::{query_rewards, query_rewards_expect_empty}, CONSUMER_1, CONSUMER_2},
};

use super::utils::{setup_app::setup_app_with_multiple_delegations, VALIDATOR};

#[test]
fn verify_rewards() {
    let (mut app_wrapper, meta_staking_addr) = setup_app_with_multiple_delegations();

    // move block year a head
    app_wrapper.update_block_seconds(60 * 60 * 24 * 365);

    // Get total amount of rewards
    let total_rewards = query_rewards(
        &app_wrapper,
        meta_staking_addr.to_string(),
        VALIDATOR.to_string(),
    );

    app_wrapper
        .execute_admin(
            meta_staking_addr.clone(),
            ExecuteMsg::WithdrawDelegatorReward {
                validator: VALIDATOR.to_string(),
            },
        )
        .unwrap();

    let new_rewards = query_rewards_expect_empty(
        &app_wrapper,
        meta_staking_addr.to_string(),
        VALIDATOR.to_string(),
    );
    assert!(new_rewards.amount.is_zero());

    // We undelegate to force reward calculation for both consumers
    app_wrapper
        .execute(
            meta_staking_addr.clone(),
            CONSUMER_1.addr(),
            ExecuteMsg::Undelegate {
                validator: VALIDATOR.to_string(),
                amount: Uint128::one(),
            },
        )
        .unwrap();

    app_wrapper
        .execute(
            meta_staking_addr.clone(),
            CONSUMER_2.addr(),
            ExecuteMsg::Undelegate {
                validator: VALIDATOR.to_string(),
                amount: Uint128::one(),
            },
        )
        .unwrap();

    app_wrapper.next_block();

    let consumer_1: ConsumerInfo = app_wrapper
        .query_smart(
            meta_staking_addr.as_str(),
            QueryMsg::Consumer {
                address: CONSUMER_1.to_string(),
            },
        )
        .unwrap();
    let consumer_2: ConsumerInfo = app_wrapper
        .query_smart(
            meta_staking_addr.as_str(),
            QueryMsg::Consumer {
                address: CONSUMER_2.to_string(),
            },
        )
        .unwrap();

    let rewards_1 = consumer_1.pending_to_u128().unwrap();
    let rewards_2 = consumer_2.pending_to_u128().unwrap();

    // make sure rewards not equal, so we confirm its not false positive test.
    // If they are equal our calculation can be off.
    assert_ne!(rewards_1, rewards_2);
    // -1 is for leftover from rounding (left as pending)
    // When we delegate we delegate not round numbers to get some leftovers.
    assert_eq!(rewards_1 + rewards_2, total_rewards.amount.u128() - 1);
}

#[test]
fn try_withdraw_no_rewards() {
    let (mut app_wrapper, meta_staking_addr) = setup_app_with_multiple_delegations();

    let err = app_wrapper
        .execute_admin(
            meta_staking_addr,
            ExecuteMsg::WithdrawDelegatorReward {
                validator: VALIDATOR.to_string(),
            },
        )
        .unwrap_err();

    assert!(matches!(err, ContractError::ZeroRewardsToSend {}));
}
