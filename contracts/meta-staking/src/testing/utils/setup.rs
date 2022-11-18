use cosmwasm_std::{coin, coins, Decimal, Uint128, Validator};
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

/// Helper return type to setup functions (make it shorter)
pub type ContractWrapperType =
    ContractWrapper<ContractError, InstantiateMsg, ExecuteMsg, QueryMsg, SudoMsg>;

///
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

/// Basic setup for unit test on a single contract
pub fn setup_contract() -> ContractWrapperType {
    // init meta-staking contract
    let mut contract = ContractWrapper::init(CONTRACT_ENTRY_POINTS, InstantiateMsg {});

    // Set the bonded denom
    contract.deps.querier.update_staking(
        NATIVE_DENOM,
        &[Validator {
            address: VALIDATOR.to_string(),
            commission: Decimal::zero(),
            max_commission: Decimal::one(),
            max_change_rate: Decimal::one(),
        }],
        &[],
    );

    // fund the contract
    contract.fund_contract(coins(100000, NATIVE_DENOM));
    contract
}

/// Using the basic setup and adding a consumer
pub fn setup_contract_with_consumer() -> ContractWrapperType {
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

/// Setup with a single delegation
pub fn setup_contract_with_delegation() -> ContractWrapperType {
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
