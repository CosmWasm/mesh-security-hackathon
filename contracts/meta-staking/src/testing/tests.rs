use cosmwasm_std::{DistributionMsg, Uint128};
use cw_multi_test::{next_block, Executor, StakingSudo, SudoMsg};

use crate::{
    error::ContractError,
    testing::{
        execute::{
            execute_delegate, execute_withdraw_rewards, execute_withdraw_rewards_should_fail, execute_withdraw_to_consumer,
        },
        ADMIN, CONSUMER, queries::{query_consumer, query_module_rewards}, CONSUMER2,
    }, state::VALIDATORS_REWARDS,
};

use super::{helpers::instantiate_setup, VALIDATOR};

#[test]
fn proper_path() {
    let (mut app, meta_staking_addr) = instantiate_setup();

    // Delegate to validator
    execute_delegate(
        &mut app,
        &meta_staking_addr,
        &CONSUMER.addr(),
        &VALIDATOR.addr(),
        Uint128::from(5555_u128)
    );

    // 2nd delegate
    execute_delegate(
        &mut app,
        &meta_staking_addr,
        &CONSUMER2.addr(),
        &VALIDATOR.addr(),
        Uint128::from(5565_u128)
    );

    let err = execute_withdraw_rewards_should_fail(
        &mut app,
        &meta_staking_addr,
        &CONSUMER.addr(),
        &VALIDATOR.addr(),
    );
    assert!(matches!(err, ContractError::ZeroRewardsToSend {}));

    // Move forward a year
    app.update_block(|block| block.time = block.time.plus_seconds(60 * 60 * 24));

    let total_rewards = query_module_rewards(&mut app, &meta_staking_addr, &VALIDATOR.addr());

    // withdraw rewards
    let res = execute_withdraw_rewards(
        &mut app,
        &meta_staking_addr,
        &CONSUMER.addr(),
        &VALIDATOR.addr(),
    );

    println!("{:?}", res);

    let new_rewards = query_module_rewards(&mut app, &meta_staking_addr, &VALIDATOR.addr());
    assert!(new_rewards.amount.is_zero());

    execute_withdraw_to_consumer(&mut app,
        &meta_staking_addr,
        &CONSUMER.addr(),
        &VALIDATOR.addr());

    // let consumer = query_consumer(&mut app, &meta_staking_addr, &CONSUMER.addr());

    // println!("{:?}", consumer);
}
