use cosmwasm_std::Addr;
use mesh_testing::unit_wrapper::UnitQuery;

use crate::msg::{Delegation, QueryMsg};

use super::utils::{
    setup::{setup_contract_with_consumer, setup_contract_with_delegation},
    CONSUMER_1,
};

#[test]
fn query_all_delegations() {
    let mut contract_wrapper = setup_contract_with_delegation();

    let all_delegations: Vec<Delegation> = contract_wrapper
        .query(QueryMsg::AllDelegations {
            consumer: CONSUMER_1.to_string(),
        })
        .unwrap();

    assert!(all_delegations.len() == 1)
}

#[test]
fn query_all_validators() {
    let mut contract_wrapper = setup_contract_with_delegation();

    let all_validators: Vec<String> = contract_wrapper
        .query(QueryMsg::AllValidators {
            consumer: CONSUMER_1.to_string(),
            start: None,
            limit: None,
        })
        .unwrap();

    assert!(all_validators.len() == 1)
}

#[test]
fn query_consumers() {
    let mut contract_wrapper = setup_contract_with_consumer();

    let consumers: Vec<Addr> = contract_wrapper
        .query(QueryMsg::Consumers {
            start: None,
            limit: None,
        })
        .unwrap();

    assert!(consumers.len() == 1)
}
