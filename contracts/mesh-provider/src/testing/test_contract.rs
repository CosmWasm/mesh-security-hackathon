use std::str::FromStr;

use cosmwasm_std::{coins, Decimal};
use mesh_testing::constants::{DELEGATOR_ADDR, REWARDS_IBC_DENOM, VALIDATOR};

use crate::testing::utils::{
    executes::execute_slash, helpers::add_validator, queries::query_validators,
};

use super::utils::{
    executes::execute_claim_rewards, helpers::add_rewards, queries::query_provider_config,
    setup::setup_with_contract,
};

#[test]
fn test_execute_slash() {
    let (mut app, mesh_provider_addr) = setup_with_contract();

    let provider_config = query_provider_config(&app, mesh_provider_addr.as_str()).unwrap();
    let mesh_slasher_addr = provider_config.slasher.unwrap();

    // Add validator to contract (mimic ibc update call)
    add_validator(&mut app, mesh_provider_addr.clone());

    let validators = query_validators(&app, mesh_provider_addr.as_str(), None, None).unwrap();
    assert_eq!(validators.validators.len(), 1);

    // Do a slash
    execute_slash(
        &mut app,
        mesh_slasher_addr.as_str(),
        mesh_provider_addr.as_str(),
        VALIDATOR,
        "0.1",
    )
    .unwrap();

    // Multiplier was 1, we slashed by 0.1 (10%), new multiplier should be 0.9.
    let validators = query_validators(&app, mesh_provider_addr.as_str(), None, None).unwrap();
    assert_eq!(
        validators.validators[0].multiplier,
        Decimal::from_str("0.9").unwrap()
    );
}

#[test]
fn test_unbond() {
    // Need to create a lockup contract, and execute stuff on it.
}

#[test]
fn test_claim_rewards() {
    let (mut app, mesh_provider_addr) = setup_with_contract();

    // Add rewards (after calculation, we added 1000 ibc_coins)
    add_rewards(&mut app, mesh_provider_addr.clone());

    execute_claim_rewards(&mut app, mesh_provider_addr.as_str(), VALIDATOR).unwrap();

    let balance = app.wrap().query_all_balances(DELEGATOR_ADDR).unwrap();

    assert_eq!(balance, coins(1000, REWARDS_IBC_DENOM))
}
