use cosmwasm_std::{
    from_binary, testing::mock_dependencies, to_binary, Decimal, Uint128, Validator, WasmMsg,
};
use mesh_ibc::{ListValidatorsResponse, StakeResponse, StdAck, UnstakeResponse};
use mesh_testing::constants::{NATIVE_DENOM, VALIDATOR};

use crate::testing::utils::helpers::{instantiate_consumer, STAKING_ADDR};

use super::utils::{
    helpers::{ack_unwrap, ibc_receive_list_validators, ibc_receive_stake, ibc_receive_unstake},
    setup::setup_with_channel,
};

#[test]
fn test_ibc_receive_list_validators() {
    let (mut deps, _) = setup_with_channel();

    // Update module to include validator
    deps.querier.update_staking(
        NATIVE_DENOM,
        &[Validator {
            address: VALIDATOR.to_string(),
            commission: Decimal::zero(),
            max_commission: Decimal::one(),
            max_change_rate: Decimal::one(),
        }],
        &[],
    );

    let res = ibc_receive_list_validators(deps.as_mut()).unwrap();
    let ack_res: ListValidatorsResponse = ack_unwrap(res.acknowledgement);

    assert_eq!(ack_res.validators, vec![VALIDATOR.to_string()],);
}

#[test]
fn test_ibc_receive_stake() {
    let (mut deps, _) = setup_with_channel();

    let res = ibc_receive_stake(deps.as_mut(), VALIDATOR, 1000, "key_1").unwrap();

    // Verify ack is success
    ack_unwrap::<StakeResponse>(res.acknowledgement.clone());
    // Verify that we send msg to meat-staking, the sent msg amount is (amount / 10) or (amount * 0.1)
    assert_eq!(
        res.messages[0].msg,
        WasmMsg::Execute {
            contract_addr: STAKING_ADDR.to_string(),
            msg: to_binary(&meta_staking::msg::ExecuteMsg::Delegate {
                validator: VALIDATOR.to_string(),
                amount: Uint128::new(100)
            })
            .unwrap(),
            funds: vec![],
        }
        .into()
    )
}

#[test]
fn test_ibc_receive_unstake() {
    let (mut deps, _) = setup_with_channel();

    let res = ibc_receive_unstake(deps.as_mut(), VALIDATOR, 1000, "key_1").unwrap();

    // Verify ack is success
    ack_unwrap::<UnstakeResponse>(res.acknowledgement.clone());
    // Verify that we send msg to meat-staking, the sent msg amount is (amount / 10) or (amount * 0.1)
    assert_eq!(
        res.messages[0].msg,
        WasmMsg::Execute {
            contract_addr: STAKING_ADDR.to_string(),
            msg: to_binary(&meta_staking::msg::ExecuteMsg::Undelegate {
                validator: VALIDATOR.to_string(),
                amount: Uint128::new(100)
            })
            .unwrap(),
            funds: vec![],
        }
        .into()
    )
}
