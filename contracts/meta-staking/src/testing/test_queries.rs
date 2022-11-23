use super::utils::{
    queries::{query_all_delegations, query_all_validators, query_consumers},
    setup::{setup_with_consumer, setup_with_multiple_delegations},
};

#[test]
fn test_query_all_delegations() {
    let (app, meta_staking_addr, mesh_consumer_addr_1, _) = setup_with_multiple_delegations();

    let delegations = query_all_delegations(
        &app,
        meta_staking_addr.as_str(),
        mesh_consumer_addr_1.as_str(),
    )
    .unwrap();

    assert!(delegations.len() == 1)
}

#[test]
fn test_query_all_validators() {
    let (app, meta_staking_addr, mesh_consumer_addr_1, _) = setup_with_multiple_delegations();

    let validators = query_all_validators(
        &app,
        meta_staking_addr.as_str(),
        mesh_consumer_addr_1.as_str(),
        None,
        None,
    )
    .unwrap();

    assert!(validators.len() == 1)
}

#[test]
fn test_query_consumers() {
    let (app, meta_staking_addr, _) = setup_with_consumer();

    let consumers = query_consumers(&app, meta_staking_addr.as_str(), None, None).unwrap();

    assert!(consumers.len() == 1)
}
