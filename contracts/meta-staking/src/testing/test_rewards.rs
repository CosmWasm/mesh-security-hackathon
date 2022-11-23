use cosmwasm_std::Uint128;

use crate::testing::utils::{
    executes::{undelegate, withdraw_rewards},
    queries::{query_consumer, query_rewards},
    setup::setup_with_multiple_delegations,
};

use mesh_testing::constants::{CREATOR_ADDR, VALIDATOR};

#[test]
fn verify_rewards() {
    let (mut app, meta_staking_addr, mesh_consumer_addr_1, mesh_consumer_addr_2) =
        setup_with_multiple_delegations();

    // move block year a head
    app.update_block(|block| block.time = block.time.plus_seconds(60 * 60 * 24 * 365));

    let total_rewards = query_rewards(&app, meta_staking_addr.as_str(), VALIDATOR).unwrap();

    withdraw_rewards(
        &mut app,
        meta_staking_addr.as_str(),
        CREATOR_ADDR,
        VALIDATOR,
    )
    .unwrap();

    // Make sure we withdrew the rewards and there are none left.
    let no_rewards = query_rewards(&app, meta_staking_addr.as_str(), VALIDATOR);
    assert!(no_rewards.is_none());

    // We undelegate to force reward calculation for both consumers
    undelegate(
        &mut app,
        meta_staking_addr.as_str(),
        mesh_consumer_addr_1.as_str(),
        VALIDATOR,
        Uint128::one(),
    )
    .unwrap();

    undelegate(
        &mut app,
        meta_staking_addr.as_str(),
        mesh_consumer_addr_2.as_str(),
        VALIDATOR,
        Uint128::one(),
    )
    .unwrap();

    // We query consumers to see how much rewards they have pending
    let consumer_1 = query_consumer(
        &app,
        meta_staking_addr.as_str(),
        mesh_consumer_addr_1.as_str(),
    )
    .unwrap();
    let consumer_2 = query_consumer(
        &app,
        meta_staking_addr.as_str(),
        mesh_consumer_addr_2.as_str(),
    )
    .unwrap();

    let rewards_1 = consumer_1.pending_to_u128().unwrap();
    let rewards_2 = consumer_2.pending_to_u128().unwrap();

    // make sure rewards not equal, so we confirm its not false positive test.
    // If they are equal our calculation can be off.
    assert_ne!(rewards_1, rewards_2);
    // -1 is for leftover from rounding (left as pending)
    // When we delegate we delegate not round numbers to get some leftovers.
    assert_eq!(rewards_1 + rewards_2, total_rewards.u128() - 1);
}
