use cosmwasm_std::{Addr, Uint128};
use cw_multi_test::next_block;

use crate::error::ContractError;

use super::helpers::{
    execute::{
        execute_delegate, execute_delegate_should_fail, execute_undelegate,
        execute_withdraw_rewards, execute_withdraw_rewards_should_fail,
    },
    instantiate::instantiate_and_add_consumer,
    queries::{query_consumer, query_delegation, query_module_rewards},
    utils::instantiate_setup,
    VALIDATOR,
};

#[test]
fn proper_path() {
    let (mut app, meta_staking_addr) = instantiate_setup();
    let consumer_1_addr = instantiate_and_add_consumer(&mut app, &meta_staking_addr);
    let consumer_2_addr = instantiate_and_add_consumer(&mut app, &meta_staking_addr);

    let consumer_1_delegations = Uint128::from(34554_u128);
    let consumer_2_delegations = Uint128::from(65446_u128);

    // Delegate to validator
    execute_delegate(
        &mut app,
        &meta_staking_addr,
        &consumer_1_addr,
        &VALIDATOR.addr(),
        consumer_1_delegations,
    );

    execute_delegate(
        &mut app,
        &meta_staking_addr,
        &consumer_2_addr,
        &VALIDATOR.addr(),
        consumer_2_delegations,
    );

    app.update_block(next_block);

    // Make sure total delegation is correct, in module and in contract
    let module_delegation = app
        .wrap()
        .query_delegation(&meta_staking_addr, &VALIDATOR.addr())
        .unwrap()
        .unwrap();
    let meta_staking_delegation_1 = query_delegation(
        &mut app,
        &meta_staking_addr,
        &consumer_1_addr,
        &VALIDATOR.addr(),
    );
    let meta_staking_delegation_2 = query_delegation(
        &mut app,
        &meta_staking_addr,
        &consumer_2_addr,
        &VALIDATOR.addr(),
    );
    assert_eq!(consumer_1_delegations, meta_staking_delegation_1);
    assert_eq!(consumer_2_delegations, meta_staking_delegation_2);
    assert_eq!(
        consumer_1_delegations + consumer_2_delegations,
        module_delegation.amount.amount
    );

    // Try to withdraw, but we have 0 rewards
    let err = execute_withdraw_rewards_should_fail(&mut app, &meta_staking_addr, &VALIDATOR.addr());
    assert!(matches!(err, ContractError::ZeroRewardsToSend {}));

    // Move forward a year to force rewards
    app.update_block(|block| block.time = block.time.plus_seconds(60 * 60 * 24 * 365));

    // Get total rewards for calculation later
    let total_rewards = query_module_rewards(&mut app, &meta_staking_addr, &VALIDATOR.addr());

    // withdraw rewards
    execute_withdraw_rewards(&mut app, &meta_staking_addr, &VALIDATOR.addr());

    // New reward should be zero because we already withdrew it.
    let new_rewards = query_module_rewards(&mut app, &meta_staking_addr, &VALIDATOR.addr());
    assert!(new_rewards.amount.is_zero());

    // Undelegate will force reward calculation for each consumer
    execute_undelegate(
        &mut app,
        &meta_staking_addr,
        &consumer_1_addr,
        &VALIDATOR.addr(),
        consumer_1_delegations,
    );

    execute_undelegate(
        &mut app,
        &meta_staking_addr,
        &consumer_2_addr,
        &VALIDATOR.addr(),
        consumer_2_delegations,
    );

    app.update_block(next_block);

    // Verify calculation works correctly
    let consumer_1 = query_consumer(&mut app, &meta_staking_addr, &consumer_1_addr);
    let consumer_2 = query_consumer(&mut app, &meta_staking_addr, &consumer_2_addr);
    let rewards_1 = consumer_1.pending_to_u128().unwrap();
    let rewards_2 = consumer_2.pending_to_u128().unwrap();

    assert_ne!(rewards_1, rewards_2); // make sure rewards not equal, so we confirm its not false positive test.
    assert_eq!(rewards_1 + rewards_2, total_rewards.amount.u128() - 1); // -1 is for leftover from rounding (left as pending)

    // Verify delegation is 0
    let module_delegation = app
        .wrap()
        .query_delegation(&meta_staking_addr, &VALIDATOR.addr())
        .unwrap();

    assert!(module_delegation.is_none());

    let meta_staking_delegation_1 = query_delegation(
        &mut app,
        &meta_staking_addr,
        &consumer_1_addr,
        &VALIDATOR.addr(),
    );
    let meta_staking_delegation_2 = query_delegation(
        &mut app,
        &meta_staking_addr,
        &consumer_2_addr,
        &VALIDATOR.addr(),
    );

    assert!(meta_staking_delegation_1.is_zero());
    assert!(meta_staking_delegation_2.is_zero());

    // Vefiry fails with NoDelegationsForValidator error
    let err = execute_withdraw_rewards_should_fail(&mut app, &meta_staking_addr, &VALIDATOR.addr());
    assert!(matches!(err, ContractError::NoDelegationsForValidator {}));
}

#[test]
fn malicious_consumer() {
    let (mut app, meta_staking_addr) = instantiate_setup();

    let malicious_consumer = Addr::unchecked("malicious_consumer");

    let err = execute_delegate_should_fail(
        &mut app,
        &meta_staking_addr,
        &malicious_consumer,
        &VALIDATOR.addr(),
        Uint128::from(100_u128),
    );

    assert!(matches!(err, ContractError::Unauthorized {}));
}
