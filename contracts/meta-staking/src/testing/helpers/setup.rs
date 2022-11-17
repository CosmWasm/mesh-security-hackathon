use cosmwasm_std::{coin, coins, testing::mock_dependencies, Uint128};
use mesh_testing::{
    unit_wrapper::{ContractEntryPoints, ContractWrapper, UnitExecute, UnitSudo},
    NATIVE_DENOM,
};

use crate::{
    contract::{execute, instantiate, query, sudo},
    error::ContractError,
    msg::{ExecuteMsg, InstantiateMsg, QueryMsg, SudoMsg},
};

use super::{CONSUMER_1, VALIDATOR};

pub const CONTRACT_ENTRY_POINTS: ContractEntryPoints<
    ContractError,
    InstantiateMsg,
    ExecuteMsg,
    QueryMsg,
    SudoMsg,
> = ContractEntryPoints {
    instantiate,
    execute,
    query,
    sudo,
};

pub fn setup_contract(
) -> ContractWrapper<ContractError, InstantiateMsg, ExecuteMsg, QueryMsg, SudoMsg> {
    let mut deps = mock_dependencies();

    // init meta-staking contract
    let mut contract = ContractWrapper::init(CONTRACT_ENTRY_POINTS, InstantiateMsg {});

    // Set the bonded denom
    contract.deps.querier.update_staking(NATIVE_DENOM, &[], &[]);

    // fund the contract
    contract.fund_contract(coins(100000, NATIVE_DENOM));
    contract
}

pub fn setup_contract_with_consumer(
) -> ContractWrapper<ContractError, InstantiateMsg, ExecuteMsg, QueryMsg, SudoMsg> {
    let mut contract_wrapper = setup_contract();

    // Add consumer
    contract_wrapper
        .sudo(SudoMsg::AddConsumer {
            consumer_address: CONSUMER_1.addr().to_string(),
            funds_available_for_staking: coin(10000, NATIVE_DENOM),
        })
        .unwrap();

    contract_wrapper
}

pub fn setup_contract_with_delegation(
) -> ContractWrapper<ContractError, InstantiateMsg, ExecuteMsg, QueryMsg, SudoMsg> {
    let mut contract_wrapper = setup_contract_with_consumer();

    // Add delegation
    contract_wrapper
        .execute(
            CONSUMER_1.addr().as_str(),
            ExecuteMsg::Delegate {
                validator: VALIDATOR.addr().to_string(),
                amount: Uint128::new(10000_u128),
            },
        )
        .unwrap();

    contract_wrapper
}
