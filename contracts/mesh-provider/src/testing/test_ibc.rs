use cosmwasm_std::{testing::mock_env, IbcChannelCloseMsg};
use mesh_ibc::IBC_APP_VERSION;
use mesh_testing::{constants::CHANNEL_ID, ibc_helpers::mock_channel};

use crate::{
    ibc::ibc_channel_close,
    testing::utils::setup_unit::{
        get_default_init_msg, ibc_connect, ibc_open, ibc_open_channel, setup_unit,
    },
    ContractError,
};

use super::utils::setup_unit::{ibc_close_channel, setup_unit_with_channel};

#[test]
fn close_channel() {
    let (mut deps, _) = setup_unit_with_channel(None);

    ibc_close_channel(deps.as_mut()).unwrap();
}

#[test]
fn test_wrong_connection() {
    let wrong_connection = "some_connection".to_string();
    let mut init_msg = get_default_init_msg(1);
    let channel = mock_channel(CHANNEL_ID, IBC_APP_VERSION);

    // Make sure we detect wrong connection
    init_msg.consumer.connection_id = wrong_connection.clone();
    let (mut deps, _) = setup_unit(Some(init_msg));
    let err = ibc_open(deps.as_mut(), channel).unwrap_err();

    assert_eq!(err, ContractError::WrongConnection(wrong_connection));
}

#[test]
fn channel_already_exists() {
    let (mut deps, _) = setup_unit_with_channel(None);

    let err = ibc_open_channel(deps.as_mut()).unwrap_err();
    assert_eq!(err, ContractError::ChannelExists(CHANNEL_ID.to_string()));

    // Test we also get channelExist on connect
    let channel = mock_channel(CHANNEL_ID, IBC_APP_VERSION);
    let err = ibc_connect(deps.as_mut(), channel).unwrap_err();
    assert_eq!(err, ContractError::ChannelExists(CHANNEL_ID.to_string()));
}

#[test]
fn try_close_wrong_channel() {
    let (mut deps, _) = setup_unit_with_channel(None);

    let some_channel = "some_channel";
    let close_msg = IbcChannelCloseMsg::new_init(mock_channel(some_channel, IBC_APP_VERSION));
    let err = ibc_channel_close(deps.as_mut(), mock_env(), close_msg).unwrap_err();

    assert_eq!(err, ContractError::UnknownChannel(some_channel.to_string()))
}
