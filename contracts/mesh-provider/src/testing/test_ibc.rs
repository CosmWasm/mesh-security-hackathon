use cosmwasm_std::{
    coin,
    testing::{mock_env, mock_info},
    to_binary, Addr, IbcChannelCloseMsg, IbcPacketReceiveMsg, Uint128,
};
use mesh_ibc::{ConsumerMsg, RewardsResponse, UpdateValidatorsResponse, IBC_APP_VERSION};
use mesh_testing::{
    addr,
    constants::{
        CHANNEL_ID, DELEGATOR_ADDR, LOCKUP_ADDR, RELAYER_ADDR, REWARDS_IBC_DENOM, VALIDATOR,
    },
    ibc_helpers::{ack_unwrap, mock_channel, mock_packet},
};

use crate::{
    contract::execute,
    ibc::{ibc_channel_close, ibc_packet_receive},
    msg::ExecuteMsg,
    state::{ValStatus, VALIDATORS},
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

#[test]
fn test_recieve_update_validators() {
    let (mut deps, _) = setup_unit_with_channel(None);
    let packet = mock_packet(
        to_binary(&ConsumerMsg::UpdateValidators {
            added: vec![VALIDATOR.to_string()],
            removed: vec![],
        })
        .unwrap(),
    );

    let res = ibc_packet_receive(
        deps.as_mut(),
        mock_env(),
        IbcPacketReceiveMsg::new(packet, addr!(RELAYER_ADDR)),
    )
    .unwrap();
    let res: UpdateValidatorsResponse = ack_unwrap(res.acknowledgement);
    assert_eq!(res, UpdateValidatorsResponse {});

    // Will fail if no validator
    VALIDATORS.load(deps.as_mut().storage, VALIDATOR).unwrap();

    // Test remove validator
    let packet = mock_packet(
        to_binary(&ConsumerMsg::UpdateValidators {
            added: vec!["new_validator".to_string()],
            removed: vec![VALIDATOR.to_string()],
        })
        .unwrap(),
    );

    let res = ibc_packet_receive(
        deps.as_mut(),
        mock_env(),
        IbcPacketReceiveMsg::new(packet, addr!(RELAYER_ADDR)),
    )
    .unwrap();
    let res: UpdateValidatorsResponse = ack_unwrap(res.acknowledgement);
    assert_eq!(res, UpdateValidatorsResponse {});

    // Should failed as we removed our validator
    let validator = VALIDATORS.load(deps.as_mut().storage, VALIDATOR).unwrap();

    assert_eq!(validator.status, ValStatus::Removed);
}

#[test]
fn test_recieve_rewards() {
    let (mut deps, _) = setup_unit_with_channel(None);
    let packet = mock_packet(
        to_binary(&ConsumerMsg::UpdateValidators {
            added: vec![VALIDATOR.to_string()],
            removed: vec![],
        })
        .unwrap(),
    );

    // Add validator
    ibc_packet_receive(
        deps.as_mut(),
        mock_env(),
        IbcPacketReceiveMsg::new(packet, addr!(RELAYER_ADDR)),
    )
    .unwrap();

    let packet = mock_packet(
        to_binary(&ConsumerMsg::Rewards {
            validator: VALIDATOR.to_string(),
            total_funds: coin(100, REWARDS_IBC_DENOM),
        })
        .unwrap(),
    );

    // add stake
    execute(
        deps.as_mut(),
        mock_env(),
        mock_info(LOCKUP_ADDR, &[]),
        ExecuteMsg::ReceiveClaim {
            owner: DELEGATOR_ADDR.to_string(),
            amount: Uint128::new(1000),
            validator: VALIDATOR.to_string(),
        },
    )
    .unwrap();

    let res = ibc_packet_receive(
        deps.as_mut(),
        mock_env(),
        IbcPacketReceiveMsg::new(packet, addr!(RELAYER_ADDR)),
    )
    .unwrap();
    let res: RewardsResponse = ack_unwrap(res.acknowledgement);
    assert_eq!(res, RewardsResponse {});
}
