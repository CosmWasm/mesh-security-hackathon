use std::str::FromStr;

use cosmwasm_std::Decimal;

use crate::testing::utils::{
    executes::execute_slash, helpers::add_validator, queries::query_validators,
};

use super::utils::{queries::query_provider_config, setup::setup_with_contract};

#[test]
fn test_execute_slash() {
    let (mut app, mesh_provider_addr) = setup_with_contract();

    let provider_config = query_provider_config(&app, mesh_provider_addr.as_str()).unwrap();
    let mesh_slasher_addr = provider_config.slasher.clone().unwrap();

    // Add validator to contract (mimic ibc update call)
    add_validator(&mut app, mesh_provider_addr.clone());

    let validators = query_validators(&app, mesh_provider_addr.as_str(), None, None).unwrap();
    assert_eq!(validators.validators.len(), 1);

    // Do a slash
    execute_slash(
        &mut app,
        mesh_slasher_addr.as_str(),
        mesh_provider_addr.as_str(),
    )
    .unwrap();

    // Multiplier was 1, we slashed by 0.1 (10%), new multiplier should be 0.9.
    let validators = query_validators(&app, mesh_provider_addr.as_str(), None, None).unwrap();
    assert_eq!(
        validators.validators[0].multiplier,
        Decimal::from_str("0.9").unwrap()
    );
}
