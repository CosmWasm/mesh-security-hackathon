use cosmwasm_std::Uint128;

use crate::ContractError;

use super::{
    assert_error,
    utils::{
        executes::{delegate, undelegate},
        queries::{query_delegation, query_module_delegation},
        setup::{setup_with_consumer, setup_with_contracts},
    },
    VALIDATOR,
};

#[test]
fn add_remove_delegations() {
    let (mut app, meta_staking_addr, mesh_consumer_addr) = setup_with_consumer();

    let delegation_amount = Uint128::new(10000_u128);

    // deleate from consumer
    delegate(
        &mut app,
        meta_staking_addr.as_str(),
        mesh_consumer_addr.as_str(),
        VALIDATOR,
        delegation_amount,
    )
    .unwrap();

    let contract_delegation = query_delegation(
        &app,
        meta_staking_addr.as_str(),
        mesh_consumer_addr.as_str(),
        VALIDATOR,
    )
    .unwrap();

    let module_delegation =
        query_module_delegation(&app, meta_staking_addr.as_str(), VALIDATOR).unwrap();

    assert_eq!(delegation_amount, contract_delegation);
    assert_eq!(delegation_amount, module_delegation.amount.amount);

    // undeleate
    undelegate(
        &mut app,
        meta_staking_addr.as_str(),
        mesh_consumer_addr.as_str(),
        VALIDATOR,
        delegation_amount,
    )
    .unwrap();

    let contract_delegation = query_delegation(
        &app,
        meta_staking_addr.as_str(),
        mesh_consumer_addr.as_str(),
        VALIDATOR,
    )
    .unwrap();

    assert_eq!(contract_delegation, Uint128::zero());
}

#[test]
fn no_consumer() {
    let (mut app, meta_staking_addr, mesh_consumer_addr) = setup_with_contracts();

    let err = delegate(
        &mut app,
        meta_staking_addr.as_str(),
        mesh_consumer_addr.as_str(),
        VALIDATOR,
        Uint128::new(1000),
    );

    assert_error!(err, ContractError::Unauthorized {});

    let err = undelegate(
        &mut app,
        meta_staking_addr.as_str(),
        mesh_consumer_addr.as_str(),
        VALIDATOR,
        Uint128::new(1000),
    );

    // TODO: this should return noConsumer error (fix in contract)
    assert_error!(err, ContractError::NoDelegationsForValidator {});
}

#[test]
fn delegate_too_much() {
    let (mut app, meta_staking_addr, mesh_consumer_addr) = setup_with_consumer();

    let err = delegate(
        &mut app,
        meta_staking_addr.as_str(),
        mesh_consumer_addr.as_str(),
        VALIDATOR,
        Uint128::new(999999999),
    );

    assert_error!(err, ContractError::NoFundsToDelegate {});
}

#[test]
fn undelegate_too_much() {
    let (mut app, meta_staking_addr, mesh_consumer_addr) = setup_with_consumer();

    delegate(
        &mut app,
        meta_staking_addr.as_str(),
        mesh_consumer_addr.as_str(),
        VALIDATOR,
        Uint128::new(1000),
    )
    .unwrap();

    let err = undelegate(
        &mut app,
        meta_staking_addr.as_str(),
        mesh_consumer_addr.as_str(),
        VALIDATOR,
        Uint128::new(999999999),
    );

    assert_error!(err, ContractError::InsufficientDelegation {});
}
