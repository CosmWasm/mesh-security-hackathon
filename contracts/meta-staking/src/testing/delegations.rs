use cosmwasm_std::Uint128;
use mesh_testing::{
    app_wrapper::{AppExecute, AppQuery},
    unit_wrapper::UnitExecute,
};

use crate::{
    error::ContractError,
    msg::{ExecuteMsg, QueryMsg},
    testing::utils::setup::setup_contract_with_consumer,
};

use super::utils::{
    setup::{setup_contract, setup_contract_with_delegation},
    setup_app::setup_app_with_consumer,
    CONSUMER_1, VALIDATOR,
};

#[test]
fn add_remove_delegations() {
    let (mut app_wrapper, meta_staking_addr) = setup_app_with_consumer();

    let delegation_amount = Uint128::new(10000_u128);

    app_wrapper
        .execute(
            meta_staking_addr.clone(),
            CONSUMER_1.addr(),
            ExecuteMsg::Delegate {
                validator: VALIDATOR.to_string(),
                amount: delegation_amount,
            },
        )
        .unwrap();

    let contract_delegation: Uint128 = app_wrapper
        .query_smart(
            meta_staking_addr.as_str(),
            QueryMsg::Delegation {
                consumer: CONSUMER_1.to_string(),
                validator: VALIDATOR.to_string(),
            },
        )
        .unwrap();
    // We delegate from meta-staking (not from consumer)
    let module_delegation = app_wrapper
        .module_querier()
        .query_delegation(meta_staking_addr.clone(), VALIDATOR.to_string())
        .unwrap()
        .unwrap();

    assert_eq!(delegation_amount, contract_delegation);
    assert_eq!(delegation_amount, module_delegation.amount.amount);

    // Undelegate
    app_wrapper
        .execute(
            meta_staking_addr.clone(),
            CONSUMER_1.addr(),
            ExecuteMsg::Undelegate {
                validator: VALIDATOR.to_string(),
                amount: delegation_amount,
            },
        )
        .unwrap();

    let contract_delegation: Uint128 = app_wrapper
        .query_smart(
            meta_staking_addr.as_str(),
            QueryMsg::Delegation {
                consumer: CONSUMER_1.to_string(),
                validator: VALIDATOR.to_string(),
            },
        )
        .unwrap();

    assert_eq!(contract_delegation, Uint128::zero());
}

#[test]
fn no_consumer() {
    let mut contract_wrapper = setup_contract();

    let err = contract_wrapper
        .execute(
            CONSUMER_1.as_str(),
            ExecuteMsg::Delegate {
                validator: VALIDATOR.addr().to_string(),
                amount: Uint128::new(1000),
            },
        )
        .unwrap_err();

    assert_eq!(err, ContractError::Unauthorized {});

    let err = contract_wrapper
        .execute(
            CONSUMER_1.as_str(),
            ExecuteMsg::Undelegate {
                validator: VALIDATOR.addr().to_string(),
                amount: Uint128::new(1000),
            },
        )
        .unwrap_err();

    // TODO: this should return noConsumer error (fix in contract)
    assert_eq!(err, ContractError::NoDelegationsForValidator {});
}

#[test]
fn delegate_too_much() {
    let mut contract_wrapper = setup_contract_with_consumer();

    let err = contract_wrapper
        .execute(
            CONSUMER_1.as_str(),
            ExecuteMsg::Delegate {
                validator: VALIDATOR.addr().to_string(),
                amount: Uint128::new(99999999),
            },
        )
        .unwrap_err();

    assert_eq!(err, ContractError::NoFundsToDelegate {});
}

#[test]
fn undelegate_too_much() {
    let mut contract_wrapper = setup_contract_with_delegation();

    let err = contract_wrapper
        .execute(
            CONSUMER_1.addr().as_str(),
            ExecuteMsg::Undelegate {
                validator: VALIDATOR.addr().to_string(),
                amount: Uint128::new(999999999_u128),
            },
        )
        .unwrap_err();

    assert_eq!(err, ContractError::InsufficientDelegation {})
}
