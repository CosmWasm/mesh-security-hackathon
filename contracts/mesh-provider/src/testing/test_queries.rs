use cosmwasm_std::Uint128;
use mesh_testing::constants::{DELEGATOR_ADDR, VALIDATOR};

use super::utils::{
    ibc_helpers::{add_stake_unit, query_account_unit, update_validator_unit},
    setup_unit::setup_unit_with_channel,
};

#[test]
fn test_query_account() {
    let (mut deps, _) = setup_unit_with_channel(None);

    update_validator_unit(deps.as_mut(), vec![VALIDATOR.to_string()], vec![]);

    add_stake_unit(deps.as_mut(), DELEGATOR_ADDR, VALIDATOR, Uint128::new(1000)).unwrap();

    let account = query_account_unit(deps.as_ref(), DELEGATOR_ADDR).unwrap();

    assert_eq!(account.staked[0].tokens, Uint128::new(1000))
}
