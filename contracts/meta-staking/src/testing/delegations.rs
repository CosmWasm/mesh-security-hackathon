use cosmwasm_std::{coin, DelegationResponse, Uint128};

use crate::{
    contract,
    msg::{ExecuteMsg, QueryMsg, SudoMsg},
    testing::helpers::setup::setup_contract_with_consumer,
};

use super::helpers::{
    contract_wrapper::{Execute, Query, Sudo},
    setup::setup_contract,
    CONSUMER_1, CONSUMER_2, NATIVE_DENOM, VALIDATOR,
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
