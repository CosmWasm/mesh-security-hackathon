use cosmwasm_std::Uint128;
use mesh_testing::constants::{VALIDATOR, DELEGATOR_ADDR};

use super::utils::{setup_unit::setup_unit_with_channel, ibc_helpers::{update_validator_unit, add_stake_unit, query_account_unit}};

#[test]
fn test_query_account() {
    let (mut deps, _) = setup_unit_with_channel(None);

    update_validator_unit(deps.as_mut(), vec![VALIDATOR.to_string()], vec![]);

    add_stake_unit(deps.as_mut(), DELEGATOR_ADDR, VALIDATOR, Uint128::new(1000)).unwrap();

    let account = query_account_unit(deps.as_ref(), DELEGATOR_ADDR).unwrap();

    assert_eq!(account.staked[0].tokens, Uint128::new(1000))
}
