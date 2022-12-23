use cosmwasm_std::{
    coin, testing::mock_env, to_binary, Addr, IbcChannelCloseMsg, IbcPacketReceiveMsg,
    IbcPacketTimeoutMsg, Uint128, WasmMsg,
};
use mesh_apis::ClaimProviderMsg;
use mesh_ibc::{
    ConsumerMsg, ProviderMsg, RewardsResponse, UpdateValidatorsResponse, IBC_APP_VERSION,
};
use mesh_testing::{
    addr,
    constants::{
        CHANNEL_ID, DELEGATOR_ADDR, LOCKUP_ADDR, RELAYER_ADDR, REWARDS_IBC_DENOM, VALIDATOR,
    },
    ibc_helpers::{ack_unwrap, mock_channel, mock_packet},
};

use crate::{
    ibc::{ibc_channel_close, ibc_packet_receive, ibc_packet_timeout},
    state::{ValStatus, LIST_VALIDATORS_MAX_RETRIES, LIST_VALIDATORS_RETRIES},
    testing::utils::ibc_helpers::{
        add_stake_unit, get_default_init_msg, ibc_connect, ibc_open, ibc_open_channel,
        query_validators_unit, update_validator_unit,
    },
    ContractError,
};

use super::utils::{
    ibc_helpers::{
        add_stake_fail_unit, ibc_close_channel, list_validators_fail_unit, list_validators_unit,
        remove_stake_fail_unit,
    },
    setup_unit::{setup_unit, setup_unit_with_channel},
};

#[test]
fn close_channel() {
    let (mut deps, _) = setup_unit_with_channel(None, CHANNEL_ID);

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
    let (mut deps, _) = setup_unit_with_channel(None, CHANNEL_ID);

    let err = ibc_open_channel(deps.as_mut(), CHANNEL_ID).unwrap_err();
    assert_eq!(err, ContractError::ChannelExists(CHANNEL_ID.to_string()));

    // Test we also get channelExist on connect
    let channel = mock_channel(CHANNEL_ID, IBC_APP_VERSION);
    let err = ibc_connect(deps.as_mut(), channel).unwrap_err();
    assert_eq!(err, ContractError::ChannelExists(CHANNEL_ID.to_string()));
}

#[test]
fn try_close_wrong_channel() {
    let (mut deps, _) = setup_unit_with_channel(None, CHANNEL_ID);

    let some_channel = "some_channel";
    let close_msg = IbcChannelCloseMsg::new_init(mock_channel(some_channel, IBC_APP_VERSION));
    let err = ibc_channel_close(deps.as_mut(), mock_env(), close_msg).unwrap_err();

    assert_eq!(err, ContractError::UnknownChannel(some_channel.to_string()))
}

#[test]
fn try_timeout() {
    let (mut deps, _) = setup_unit_with_channel(None, CHANNEL_ID);

    // Test list validators, returns 1 msg (Retry)
    let packet = mock_packet(to_binary(&ProviderMsg::ListValidators {}).unwrap());
    let msg = IbcPacketTimeoutMsg::new(packet, addr!(RELAYER_ADDR));
    let res = ibc_packet_timeout(deps.as_mut(), mock_env(), msg).unwrap();
    assert_eq!(res.messages.len(), 1);

    // Test stake, need to return 1 msg (unbond)
    let packet = mock_packet(
        to_binary(&ProviderMsg::Stake {
            validator: VALIDATOR.to_string(),
            amount: Uint128::new(1000),
            key: DELEGATOR_ADDR.to_string(),
        })
        .unwrap(),
    );
    let msg = IbcPacketTimeoutMsg::new(packet, addr!(RELAYER_ADDR));
    let res = ibc_packet_timeout(deps.as_mut(), mock_env(), msg).unwrap();
    assert_eq!(res.messages.len(), 1);

    // Nothing to return, just make sure it passed.
    let packet = mock_packet(
        to_binary(&ProviderMsg::Unstake {
            validator: VALIDATOR.to_string(),
            amount: Uint128::new(1000),
            key: DELEGATOR_ADDR.to_string(),
        })
        .unwrap(),
    );
    let msg = IbcPacketTimeoutMsg::new(packet, addr!(RELAYER_ADDR));
    ibc_packet_timeout(deps.as_mut(), mock_env(), msg).unwrap();
}

#[test]
fn try_recieve_without_channel() {
    let (mut deps, _) = setup_unit(None);
    let err = update_validator_unit(
        deps.as_mut(),
        vec!["new_validator".to_string()],
        vec![VALIDATOR.to_string()],
    )
    .unwrap_err();
    assert_eq!(err, ContractError::UnknownChannel(CHANNEL_ID.to_string()));

    // Open channel, and use different channel when doing recieve
    ibc_open_channel(deps.as_mut(), "some_channel").unwrap();
    let err = update_validator_unit(
        deps.as_mut(),
        vec!["new_validator".to_string()],
        vec![VALIDATOR.to_string()],
    )
    .unwrap_err();
    assert_eq!(err, ContractError::UnknownChannel(CHANNEL_ID.to_string()));
}

#[test]
fn test_recieve_update_validators() {
    let (mut deps, _) = setup_unit_with_channel(None, CHANNEL_ID);

    let res = update_validator_unit(deps.as_mut(), vec![VALIDATOR.to_string()], vec![]).unwrap();
    let res: UpdateValidatorsResponse = ack_unwrap(res.acknowledgement);
    assert_eq!(res, UpdateValidatorsResponse {});

    // Will fail if no validator
    query_validators_unit(deps.as_ref(), VALIDATOR).unwrap();

    // Test remove validator
    let res = update_validator_unit(
        deps.as_mut(),
        vec!["new_validator".to_string()],
        vec![VALIDATOR.to_string()],
    )
    .unwrap();
    let res: UpdateValidatorsResponse = ack_unwrap(res.acknowledgement);
    assert_eq!(res, UpdateValidatorsResponse {});

    // Should failed as we removed our validator
    let validator = query_validators_unit(deps.as_ref(), VALIDATOR).unwrap();

    assert_eq!(validator.status, ValStatus::Removed);
}

#[test]
fn test_recieve_rewards() {
    let (mut deps, _) = setup_unit_with_channel(None, CHANNEL_ID);

    // Add validator
    update_validator_unit(deps.as_mut(), vec![VALIDATOR.to_string()], vec![]).unwrap();

    let packet = mock_packet(
        to_binary(&ConsumerMsg::Rewards {
            validator: VALIDATOR.to_string(),
            total_funds: coin(100, REWARDS_IBC_DENOM),
        })
        .unwrap(),
    );

    // add stake
    add_stake_unit(deps.as_mut(), DELEGATOR_ADDR, VALIDATOR, Uint128::new(1000)).unwrap();

    let res = ibc_packet_receive(
        deps.as_mut(),
        mock_env(),
        IbcPacketReceiveMsg::new(packet, addr!(RELAYER_ADDR)),
    )
    .unwrap();
    let res: RewardsResponse = ack_unwrap(res.acknowledgement);
    assert_eq!(res, RewardsResponse {});
}

#[test]
fn test_recieve_rewards_failing() {
    let (mut deps, _) = setup_unit_with_channel(None, CHANNEL_ID);

    // Add validator
    update_validator_unit(deps.as_mut(), vec![VALIDATOR.to_string()], vec![]).unwrap();

    // Try unknown validator
    let err_packet = mock_packet(
        to_binary(&ConsumerMsg::Rewards {
            validator: "some_validator".to_string(),
            total_funds: coin(100, REWARDS_IBC_DENOM),
        })
        .unwrap(),
    );
    let err = ibc_packet_receive(
        deps.as_mut(),
        mock_env(),
        IbcPacketReceiveMsg::new(err_packet, addr!(RELAYER_ADDR)),
    )
    .unwrap_err();
    assert_eq!(
        err,
        ContractError::UnknownValidator("some_validator".to_string())
    );

    // add stake zero
    add_stake_unit(deps.as_mut(), DELEGATOR_ADDR, VALIDATOR, Uint128::zero()).unwrap();

    let err_packet = mock_packet(
        to_binary(&ConsumerMsg::Rewards {
            validator: VALIDATOR.to_string(),
            total_funds: coin(100, REWARDS_IBC_DENOM),
        })
        .unwrap(),
    );
    let err = ibc_packet_receive(
        deps.as_mut(),
        mock_env(),
        IbcPacketReceiveMsg::new(err_packet, addr!(RELAYER_ADDR)),
    )
    .unwrap_err();
    assert_eq!(err, ContractError::NoStakedTokens(VALIDATOR.to_string()));
}

#[test]
fn test_list_validators() {
    let (mut deps, _) = setup_unit_with_channel(None, CHANNEL_ID);

    list_validators_unit(deps.as_mut()).unwrap();
    let validator = query_validators_unit(deps.as_ref(), VALIDATOR).unwrap();
    assert_eq!(validator.status, ValStatus::Active)
}

#[test]
fn test_list_validators_fail() {
    let (mut deps, _) = setup_unit_with_channel(None, CHANNEL_ID);

    let res = list_validators_fail_unit(deps.as_mut()).unwrap();
    let retries_num = LIST_VALIDATORS_RETRIES.load(deps.as_ref().storage).unwrap();

    assert_eq!(res.messages.len(), 1); // We do a retry
    assert_eq!(retries_num, LIST_VALIDATORS_MAX_RETRIES - 1);

    // do 4 more retries
    list_validators_fail_unit(deps.as_mut()).unwrap();
    list_validators_fail_unit(deps.as_mut()).unwrap();
    list_validators_fail_unit(deps.as_mut()).unwrap();
    list_validators_fail_unit(deps.as_mut()).unwrap();

    // on last retry
    let res = list_validators_fail_unit(deps.as_mut()).unwrap();
    let retries_num = LIST_VALIDATORS_RETRIES.load(deps.as_ref().storage).unwrap();

    // We retried 5 times, we should stop retrying and reset count
    assert_eq!(res.messages.len(), 0); // We do a retry
    assert_eq!(retries_num, LIST_VALIDATORS_MAX_RETRIES);
}

#[test]
fn test_add_stake_fail() {
    let (mut deps, _) = setup_unit_with_channel(None, CHANNEL_ID);

    let res =
        add_stake_fail_unit(deps.as_mut(), DELEGATOR_ADDR, VALIDATOR, Uint128::new(1000)).unwrap();

    assert_eq!(
        res.messages[0].msg,
        WasmMsg::Execute {
            contract_addr: LOCKUP_ADDR.to_string(),
            msg: to_binary(&ClaimProviderMsg::SlashClaim {
                owner: DELEGATOR_ADDR.to_string(),
                amount: Uint128::new(1000)
            })
            .unwrap(),
            funds: vec![]
        }
        .into()
    );
}

#[test]
fn test_remove_stake_fail() {
    let (mut deps, _) = setup_unit_with_channel(None, CHANNEL_ID);

    let res = remove_stake_fail_unit(deps.as_mut(), DELEGATOR_ADDR, VALIDATOR, Uint128::new(1000))
        .unwrap();

    assert_eq!(res.events[0].ty, "failed_unstake")
}
