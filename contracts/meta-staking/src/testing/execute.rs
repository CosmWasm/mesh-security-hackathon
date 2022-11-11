use cosmwasm_std::{Addr, Uint128};
use cw_multi_test::{next_block, App, Executor, AppResponse};

use crate::{error::ContractError, msg::ExecuteMsg, testing::queries::query_delegation};

use super::ADMIN;

pub fn execute_delegate(app: &mut App, contract: &Addr, from: &Addr, validator: &Addr, amount: Uint128) {
    app.execute_contract(
        from.clone(),
        contract.clone(),
        &ExecuteMsg::Delegate {
            validator: validator.into(),
            amount,
        },
        &[],
    )
    .unwrap();

    // Verify we got it right.
    app.update_block(next_block);

    let meta_staking_delegation = query_delegation(app, &contract, &from, &validator);
    assert_eq!(amount, meta_staking_delegation);
}

pub fn execute_withdraw_rewards(app: &mut App, contract: &Addr, consumer: &Addr, validator: &Addr) -> AppResponse{
    app.execute_contract(
        ADMIN.addr(),
        contract.clone(),
        &ExecuteMsg::WithdrawDelegatorReward {
            validator: validator.to_string(),
        },
        &[],
    )
    .unwrap()
}

pub fn execute_withdraw_rewards_should_fail(
    app: &mut App,
    contract: &Addr,
    _consumer: &Addr,
    validator: &Addr,
) -> ContractError {
    app.execute_contract(
        ADMIN.addr(),
        contract.clone(),
        &ExecuteMsg::WithdrawDelegatorReward {
            validator: validator.to_string(),
        },
        &[],
    )
    .unwrap_err()
    .downcast()
    .unwrap()
}

pub fn execute_withdraw_to_consumer(
    app: &mut App,
    contract: &Addr,
    consumer: &Addr,
    validator: &Addr,
) {
    let res = app.execute_contract(
        ADMIN.addr(),
        contract.clone(),
        &ExecuteMsg::WithdrawToCostumer {
            consumer: consumer.to_string(),
            validator: validator.to_string(),
        },
        &[],
    )
    .unwrap();
}
