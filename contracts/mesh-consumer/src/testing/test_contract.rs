use cosmwasm_std::{
    coin,
    testing::{mock_env, mock_info},
    to_binary, IbcMsg,
};
use mesh_ibc::ConsumerMsg;
use mesh_testing::constants::{NATIVE_DENOM, VALIDATOR};

use crate::{
    ibc::build_timeout,
    testing::utils::{
        helpers::{execute_receive_rewards, CHANNEL_ID, STAKING_ADDR},
        setup::setup_with_channel,
    },
};

#[test]
fn recieve_rewards() {
    let (mut deps, _) = setup_with_channel();

    // test execute receive rewards
    let coin = coin(1000, NATIVE_DENOM);
    let info = mock_info(STAKING_ADDR, &[coin.clone()]);
    let res = execute_receive_rewards(deps.as_mut(), info, VALIDATOR).unwrap();

    assert_eq!(
        res.messages[0].msg,
        IbcMsg::SendPacket {
            channel_id: CHANNEL_ID.to_string(),
            data: to_binary(&ConsumerMsg::Rewards {
                validator: VALIDATOR.to_string(),
                total_funds: coin,
            })
            .unwrap(),
            timeout: build_timeout(deps.as_ref(), &mock_env()).unwrap(),
        }
        .into()
    )
}
