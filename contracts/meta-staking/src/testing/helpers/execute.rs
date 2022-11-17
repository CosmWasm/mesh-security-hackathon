use cosmwasm_std::{Addr, Uint128};
use cw_multi_test::{App, AppResponse, Executor};

use crate::{error::ContractError, msg::ExecuteMsg};

use super::super::helpers::ADMIN;

pub fn execute_delegate(
    app: &mut App,
    contract: &Addr,
    from: &Addr,
    validator: &Addr,
    amount: Uint128,
) {
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
}

pub fn execute_delegate_should_fail(
    app: &mut App,
    contract: &Addr,
    from: &Addr,
    validator: &Addr,
    amount: Uint128,
) -> ContractError {
    app.execute_contract(
        from.clone(),
        contract.clone(),
        &ExecuteMsg::Delegate {
            validator: validator.into(),
            amount,
        },
        &[],
    )
    .unwrap_err()
    .downcast()
    .unwrap()
}

pub fn execute_withdraw_rewards(app: &mut App, contract: &Addr, validator: &Addr) -> AppResponse {
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

pub fn execute_undelegate(
    app: &mut App,
    contract: &Addr,
    from: &Addr,
    validator: &Addr,
    amount: Uint128,
) {
    app.execute_contract(
        from.clone(),
        contract.clone(),
        &ExecuteMsg::Undelegate {
            validator: validator.into(),
            amount,
        },
        &[],
    )
    .unwrap();
}
