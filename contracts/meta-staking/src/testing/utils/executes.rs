use anyhow::Result as AnyResult;
use cosmwasm_std::{coin, Addr, Uint128};
use cw_multi_test::{App, AppResponse, Executor};

use crate::{
    msg::{ExecuteMsg, SudoMsg},
    testing::NATIVE_DENOM,
};

// Shorthand for an unchecked address.
macro_rules! addr {
    ($x:expr ) => {
        Addr::unchecked($x)
    };
}

pub fn delegate(
    app: &mut App,
    contract_addr: Addr,
    sender: &str,
    validator: &str,
    amount: Uint128,
) -> AnyResult<AppResponse> {
    app.execute_contract(
        addr!(sender),
        contract_addr.clone(),
        &ExecuteMsg::Delegate {
            validator: validator.to_string(),
            amount,
        },
        &[],
    )
}

pub fn undelegate(
    app: &mut App,
    contract_addr: Addr,
    sender: &str,
    validator: &str,
    amount: Uint128,
) -> AnyResult<AppResponse> {
    app.execute_contract(
        addr!(sender),
        contract_addr.clone(),
        &ExecuteMsg::Undelegate {
            validator: validator.to_string(),
            amount,
        },
        &[],
    )
}

pub fn withdraw_rewards(
    app: &mut App,
    contract_addr: Addr,
    sender: &str,
    validator: &str,
) -> AnyResult<AppResponse> {
    app.execute_contract(
        addr!(sender),
        contract_addr.clone(),
        &ExecuteMsg::WithdrawDelegatorReward {
            validator: validator.to_string(),
        },
        &[],
    )
}

pub fn add_consumer(
    app: &mut App,
    contract_addr: Addr,
    sender: &str,
    consumer_addr: &str,
    funds_avaiable: u128,
) -> AnyResult<AppResponse> {
    let sudo_msg = SudoMsg::AddConsumer {
        consumer_address: consumer_addr.to_string(),
        funds_available_for_staking: coin(funds_avaiable, NATIVE_DENOM),
    };

    app.execute_contract(
        addr!(sender),
        contract_addr.clone(),
        &ExecuteMsg::Sudo(sudo_msg),
        &[],
    )
}

// TODO: withdraw to consumer end with IBC call which is not supported by cw-multi-test
pub fn withdraw_to_consumer(
    app: &mut App,
    contract_addr: Addr,
    sender: &str,
    consumer: &str,
    validator: &str,
) -> AnyResult<AppResponse> {
    unimplemented!()
    // app.execute_contract(
    //     addr!(sender),
    //     contract_addr.clone(),
    //     &ExecuteMsg::WithdrawToCostumer { consumer: consumer.to_string(), validator: validator.to_string() },
    //     &[],
    // )
}
