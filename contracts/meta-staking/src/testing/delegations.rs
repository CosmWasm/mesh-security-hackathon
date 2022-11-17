use cosmwasm_std::Uint128;
use mesh_testing::unit_wrapper::{UnitExecute, UnitQuery};

use crate::{
    error::ContractError,
    msg::{ExecuteMsg, QueryMsg},
    testing::helpers::setup::setup_contract_with_consumer,
};

use super::helpers::{
    setup::{setup_contract, setup_contract_with_delegation},
    CONSUMER_1, VALIDATOR,
};

#[test]
fn add_remove_delegations() {
    let mut contract_wrapper = setup_contract_with_consumer();

    let delegation_amount = Uint128::new(10000_u128);

    // Delegate from consumer
    contract_wrapper
        .execute(
            CONSUMER_1.addr().as_str(),
            ExecuteMsg::Delegate {
                validator: VALIDATOR.addr().to_string(),
                amount: delegation_amount,
            },
        )
        .unwrap();

    // Test delegation amounts
    let contract_delegation = contract_wrapper
        .query(QueryMsg::Delegation {
            consumer: CONSUMER_1.to_string(),
            validator: VALIDATOR.to_string(),
        })
        .unwrap::<Uint128>();

    assert_eq!(delegation_amount, contract_delegation);

    // Undelegate from consumer
    contract_wrapper
        .execute(
            CONSUMER_1.addr().as_str(),
            ExecuteMsg::Undelegate {
                validator: VALIDATOR.addr().to_string(),
                amount: delegation_amount,
            },
        )
        .unwrap();

    // Verify we have 0 delegations
    let contract_delegation = contract_wrapper
        .query(QueryMsg::Delegation {
            consumer: CONSUMER_1.to_string(),
            validator: VALIDATOR.to_string(),
        })
        .unwrap::<Uint128>();

    assert_eq!(contract_delegation, Uint128::zero());
}

#[test]
fn no_consumer() {
    let mut contract_wrapper = setup_contract();

    let err = contract_wrapper
        .execute(
            &CONSUMER_1.to_string(),
            ExecuteMsg::Delegate {
                validator: VALIDATOR.addr().to_string(),
                amount: Uint128::new(1000),
            },
        )
        .unwrap_err();

    assert_eq!(err, ContractError::Unauthorized {});

    let err = contract_wrapper
        .execute(
            &CONSUMER_1.to_string(),
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
            &CONSUMER_1.to_string(),
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
