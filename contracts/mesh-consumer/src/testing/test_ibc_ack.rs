use cosmwasm_std::{coin, testing::mock_env, IbcAcknowledgement, IbcMsg};
use mesh_ibc::RewardsResponse;
use mesh_testing::constants::{NATIVE_DENOM, VALIDATOR};

use crate::{
    ibc::build_timeout,
    testing::utils::helpers::{ICS20_CHANNEL_ID, REMOTE_PORT},
};

use super::utils::{executes::ibc_ack_rewards, helpers::to_ack_success, setup::setup_with_channel};

#[test]
fn test_ibc_ack_rewards() {
    let (mut deps, _) = setup_with_channel();

    // We test a successful ack
    let ack = IbcAcknowledgement::new(to_ack_success(RewardsResponse {}));

    let res = ibc_ack_rewards(deps.as_mut(), VALIDATOR, 100, ack).unwrap();

    assert_eq!(
        res.messages[0].msg,
        IbcMsg::Transfer {
            channel_id: ICS20_CHANNEL_ID.to_string(),
            to_address: REMOTE_PORT
                .to_string()
                .split('.')
                .last()
                .unwrap()
                .to_string(), // port - prefix
            amount: coin(100, NATIVE_DENOM),
            timeout: build_timeout(deps.as_ref(), &mock_env()).unwrap(),
        }
        .into()
    );
}

#[test]
fn test_ibc_update_validators() {
    unimplemented!()
}
