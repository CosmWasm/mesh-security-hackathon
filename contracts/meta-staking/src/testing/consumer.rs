use cosmwasm_std::{coin, Decimal, StdError, Uint128};
use mesh_testing::{
    unit_wrapper::{UnitQuery, UnitSudo},
    NATIVE_DENOM,
};

use crate::{
    error::ContractError,
    msg::{QueryMsg, SudoMsg},
    state::ConsumerInfo,
    testing::utils::setup::setup_contract,
};

use super::utils::CONSUMER_1;

#[test]
fn add_and_remove_consumer() {
    let mut contract_wrapper = setup_contract();

    // add consumer
    contract_wrapper
        .sudo(SudoMsg::AddConsumer {
            consumer_address: CONSUMER_1.addr().to_string(),
            funds_available_for_staking: coin(10000, NATIVE_DENOM),
        })
        .unwrap();

    // Get consumer from contract
    let consumer: ConsumerInfo = contract_wrapper
        .query(QueryMsg::Consumer {
            address: CONSUMER_1.addr().to_string(),
        })
        .unwrap();

    // Test consumer holds the correct data
    assert_eq!(consumer.available_funds, Uint128::new(10000_u128));
    assert_eq!(consumer.rewards.pending, Decimal::zero());

    // remove consumer
    contract_wrapper
        .sudo(SudoMsg::RemoveConsumer {
            consumer_address: CONSUMER_1.addr().to_string(),
        })
        .unwrap();

    // try get consumer, expect error
    let err = contract_wrapper
        .query(QueryMsg::Consumer {
            address: CONSUMER_1.addr().to_string(),
        })
        .unwrap_err();

    assert_eq!(
        err,
        StdError::NotFound {
            kind: "meta_staking::state::ConsumerInfo".to_string()
        }
    );
}

#[test]
fn consumer_already_exists() {
    let mut contract_wrapper = setup_contract();

    contract_wrapper
        .sudo(SudoMsg::AddConsumer {
            consumer_address: CONSUMER_1.addr().to_string(),
            funds_available_for_staking: coin(10000, NATIVE_DENOM),
        })
        .unwrap();

    let err = contract_wrapper
        .sudo(SudoMsg::AddConsumer {
            consumer_address: CONSUMER_1.addr().to_string(),
            funds_available_for_staking: coin(10000, NATIVE_DENOM),
        })
        .unwrap_err();

    assert_eq!(err, ContractError::ConsumerAlreadyExists {});
}

#[test]
fn consumer_not_enough_funds() {
    let mut contract_wrapper = setup_contract();

    let err = contract_wrapper
        .sudo(SudoMsg::AddConsumer {
            consumer_address: CONSUMER_1.addr().to_string(),
            funds_available_for_staking: coin(9999999999, NATIVE_DENOM),
        })
        .unwrap_err();

    assert_eq!(err, ContractError::NotEnoughFunds {});
}

#[test]
fn consumer_remove_not_exists() {
    let mut contract_wrapper = setup_contract();

    let err = contract_wrapper
        .sudo(SudoMsg::RemoveConsumer {
            consumer_address: CONSUMER_1.addr().to_string(),
        })
        .unwrap_err();

    assert_eq!(err, ContractError::NoConsumer {});
}
