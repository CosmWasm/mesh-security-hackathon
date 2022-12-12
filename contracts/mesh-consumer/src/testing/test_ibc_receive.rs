use cosmwasm_std::{
    testing::mock_env, to_binary, Addr, Decimal, IbcPacketReceiveMsg, Uint128, Validator, WasmMsg,
};
use mesh_apis::StakingExecuteMsg;
use mesh_ibc::{ListValidatorsResponse, ProviderMsg, StakeResponse, UnstakeResponse};
use mesh_testing::{
    addr,
    constants::{NATIVE_DENOM, VALIDATOR, RELAYER_ADDR}, ibc_helpers::{ack_unwrap, mock_packet},
};

use crate::{ibc::ibc_packet_receive, ContractError};

use super::utils::{
    executes::ibc_receive_list_validators,
    executes::{ibc_receive_stake, ibc_receive_unstake},
    helpers::STAKING_ADDR,
    setup::setup_with_channel,
};

#[test]
fn test_ibc_receive_list_validators() {
    let (mut deps, _) = setup_with_channel(None);

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
    let (mut deps, _) = setup_with_channel(None);

    let res = ibc_receive_stake(deps.as_mut(), VALIDATOR, 1000, "key_1").unwrap();

    // Verify ack is success
    ack_unwrap::<StakeResponse>(res.acknowledgement.clone());
    // Verify that we send msg to meat-staking, the sent msg amount is (amount / 10) or (amount * 0.1)
    assert_eq!(
        res.messages[0].msg,
        WasmMsg::Execute {
            contract_addr: STAKING_ADDR.to_string(),
            msg: to_binary(&StakingExecuteMsg::Delegate {
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
    let (mut deps, _) = setup_with_channel(None);

    let res = ibc_receive_unstake(deps.as_mut(), VALIDATOR, 1000, "key_1").unwrap();

    // Verify ack is success
    ack_unwrap::<UnstakeResponse>(res.acknowledgement.clone());
    // Verify that we send msg to meat-staking, the sent msg amount is (amount / 10) or (amount * 0.1)
    assert_eq!(
        res.messages[0].msg,
        WasmMsg::Execute {
            contract_addr: STAKING_ADDR.to_string(),
            msg: to_binary(&StakingExecuteMsg::Undelegate {
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
fn test_ibc_receive_wrong_channel() {
    let (mut deps, _) = setup_with_channel(None);
    let mut packet = mock_packet(to_binary(&ProviderMsg::ListValidators {}).unwrap());
    packet.dest.channel_id = "some_channel".to_string();

    let err = ibc_packet_receive(
        deps.as_mut(),
        mock_env(),
        IbcPacketReceiveMsg::new(packet, addr!(RELAYER_ADDR)),
    )
    .unwrap_err();
    assert_eq!(
        err,
        ContractError::UnknownChannel("some_channel".to_string())
    )
}
