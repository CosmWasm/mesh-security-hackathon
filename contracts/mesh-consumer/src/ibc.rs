#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    from_slice, to_binary, Coin, Deps, DepsMut, Env, Ibc3ChannelOpenResponse, IbcBasicResponse,
    IbcChannelCloseMsg, IbcChannelConnectMsg, IbcChannelOpenMsg, IbcMsg, IbcPacketAckMsg,
    IbcPacketReceiveMsg, IbcPacketTimeoutMsg, IbcReceiveResponse, IbcTimeout, Uint128, WasmMsg,
};

use cw_utils::{parse_execute_response_data, parse_reply_execute_data, MsgExecuteContractResponse};

use mesh_apis::CallbackDataResponse;
use mesh_ibc::{check_order, check_version, ConsumerMsg, ProviderMsg, StdAck};
use meta_staking::msg::ExecuteMsg as MetaStakingExecuteMsg;

use crate::error::ContractError;
use crate::state::{CHANNEL, CONFIG, PACKET_LIFETIME, STAKED, VALIDATORS};

const STAKE_CALLBACK_ID: u64 = 1;
const UNSTAKE_CALLBACK_ID: u64 = 2;
const WITHDRAW_REWARDS_CALLBACK_ID: u64 = 3;

pub fn build_timeout(deps: Deps, env: &Env) -> Result<IbcTimeout, ContractError> {
    let packet_time = PACKET_LIFETIME.load(deps.storage)?;
    let time = env.block.time.plus_seconds(packet_time);
    Ok(IbcTimeout::with_timestamp(time))
}

#[cfg_attr(not(feature = "library"), entry_point)]
/// enforces ordering and versioning constraints
pub fn ibc_channel_open(
    deps: DepsMut,
    _env: Env,
    msg: IbcChannelOpenMsg,
) -> Result<Option<Ibc3ChannelOpenResponse>, ContractError> {
    // ensure we have no other channels currently
    if let Some(chan) = CHANNEL.may_load(deps.storage)? {
        return Err(ContractError::ChannelExists(chan));
    }

    // check the handshake order/version is correct
    let channel = msg.channel();
    check_order(&channel.order)?;
    check_version(&channel.version)?;
    if let Some(counter_version) = msg.counterparty_version() {
        check_version(counter_version)?;
    }

    // ensure the remote connection / port is authorized
    let cfg = CONFIG.load(deps.storage)?;
    if cfg.provider.connection_id != channel.connection_id {
        return Err(ContractError::WrongConnection(cfg.provider.connection_id));
    }

    if cfg.provider.port_id != channel.counterparty_endpoint.port_id {
        return Err(ContractError::WrongPort(cfg.provider.port_id));
    }

    Ok(None)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn ibc_channel_connect(
    deps: DepsMut,
    _env: Env,
    msg: IbcChannelConnectMsg,
) -> Result<IbcBasicResponse, ContractError> {
    let channel = msg.channel();
    let channel_id = &channel.endpoint.channel_id;

    // save the channel id for future use
    match CHANNEL.may_load(deps.storage)? {
        Some(chan) => return Err(ContractError::ChannelExists(chan)),
        None => CHANNEL.save(deps.storage, channel_id)?,
    };

    Ok(IbcBasicResponse::new())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn ibc_channel_close(
    deps: DepsMut,
    _env: Env,
    msg: IbcChannelCloseMsg,
) -> Result<IbcBasicResponse, ContractError> {
    let channel = msg.channel();
    let channel_id = &channel.endpoint.channel_id;

    // let's ensure this is really closed by same channel we previously connected (paranoia?)
    // then delete from store
    let channel = CHANNEL.load(deps.storage)?;
    if &channel == channel_id {
        CHANNEL.remove(deps.storage);
    } else {
        return Err(ContractError::UnknownChannel(channel_id.clone()));
    }

    Ok(IbcBasicResponse::new())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn ibc_packet_receive(
    deps: DepsMut,
    env: Env,
    msg: IbcPacketReceiveMsg,
) -> Result<IbcReceiveResponse, ContractError> {
    // paranoia: ensure it was sent on proper channel
    let caller = msg.packet.dest.channel_id;
    if CHANNEL.load(deps.storage)? != caller {
        return Err(ContractError::UnknownChannel(caller));
    }

    let msg: ProviderMsg = from_slice(&msg.packet.data)?;
    match msg {
        ProviderMsg::ListValidators {} => receive_list_validators(deps),
        ProviderMsg::Stake {
            validator,
            amount,
            delegator_addr,
        } => receive_stake(deps, validator, amount, delegator_addr),
        ProviderMsg::Unstake {
            validator,
            amount,
            delegator_addr,
        } => receive_unstake(deps, validator, amount, delegator_addr),
        ProviderMsg::WithdrawRewards {
            validator,
            delegator_addr,
        } => receive_withdraw_rewards(deps, env, validator, delegator_addr),
    }
}

pub fn receive_list_validators(deps: DepsMut) -> Result<IbcReceiveResponse, ContractError> {
    let validators = deps
        .querier
        .query_all_validators()?
        .into_iter()
        .map(|x| x.address)
        .collect();
    let ack = StdAck::success(mesh_ibc::ListValidatorsResponse { validators });

    Ok(IbcReceiveResponse::new().set_ack(ack))
}

pub fn receive_stake(
    deps: DepsMut,
    validator: String,
    amount: Uint128,
    staker: String,
) -> Result<IbcReceiveResponse, ContractError> {
    let config = CONFIG.load(deps.storage)?;

    // Convert remote token to local token
    let amount = amount * config.remote_to_local_exchange_rate;

    let msg = SubMsg::reply_on_success(
        WasmMsg::Execute {
            contract_addr: config.meta_staking_contract_address.to_string(),
            msg: to_binary(&MetaStakingExecuteMsg::Delegate {
                validator,
                staker,
                amount,
            })?,
            funds: vec![],
        },
        STAKE_CALLBACK_ID,
    );

    Ok(IbcReceiveResponse::new().add_submessage(msg).set_ack(StdAck::success("1")))
}

pub fn receive_unstake(
    deps: DepsMut,
    validator: String,
    amount: Uint128,
    staker: String,
) -> Result<IbcReceiveResponse, ContractError> {
    let config = CONFIG.load(deps.storage)?;

    // Convert remote token to local token
    let amount = amount * config.remote_to_local_exchange_rate;

    let msg = SubMsg::reply_on_success(
        WasmMsg::Execute {
            contract_addr: config.meta_staking_contract_address.to_string(),
            msg: to_binary(&MetaStakingExecuteMsg::Undelegate {
                validator,
                staker,
                amount,
            })?,
            funds: vec![],
        },
        UNSTAKE_CALLBACK_ID,
    );

    Ok(IbcReceiveResponse::new().add_submessage(msg))
}

pub fn receive_withdraw_rewards(
    deps: DepsMut,
    env: Env,
    validator: String,
    staker: String,
) -> Result<IbcReceiveResponse, ContractError> {
    let config = CONFIG.load(deps.storage)?;

    let msg = SubMsg::reply_on_success(
        WasmMsg::Execute {
            contract_addr: config.meta_staking_contract_address.to_string(),
            msg: to_binary(&MetaStakingExecuteMsg::WithdrawToCostumer {
                consumer: env.contract.address,
                staker,
                validator,
            })?,
            funds: vec![],
        },
        WITHDRAW_REWARDS_CALLBACK_ID,
    );

    Ok(IbcReceiveResponse::new().add_submessage(msg))
}

fn verify_callback_data(data: Binary) -> Result<CallbackDataResponse, StdError> {
    from_slice(&data)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(deps: DepsMut, env: Env, reply: Reply) -> Result<Response, ContractError> {
    // on reply, we calculate rewards based on recent rewards from validator, and old stake
    // then we update storage with new stake (after accurate rewards calculation)
    let result = parse_reply_execute_data(reply.clone())?;

    if result.data.is_none() {
        return Err(ContractError::AckDataIsNone {});
    }
    let data = result.data.unwrap();

    match reply.id {
        STAKE_CALLBACK_ID => reply_stake_callback(deps, data),
        UNSTAKE_CALLBACK_ID => reply_unstake_callback(deps, data),
        WITHDRAW_REWARDS_CALLBACK_ID => {
            reply_withdraw_rewards_callback(deps, env, data)
        }
        _ => Err(ContractError::InvalidReplyId(reply.id)),
    }
}

pub fn reply_stake_callback(
    deps: DepsMut,
    data: Binary,
) -> Result<Response, ContractError> {
    let CallbackDataResponse {
        validator,
        staker,
        stake_amount,
        rewards,
    } = verify_callback_data(data)?;

    // We trust provider tested validator exists, so we can safely load default validator if we
    // haven't staked for this validator before.
    let mut val = VALIDATORS
        .may_load(deps.storage, &validator)?
        .unwrap_or_default();
    let mut stake = STAKED
        .may_load(deps.storage, (&staker, &validator))?
        .unwrap_or_default();

    // calculate validator rewards and save.
    val.calc_rewards(rewards.amount)?;

    // update Stake and calculate rewards of delegator (key)
    stake.calc_pending_rewards(val.rewards_per_token, val.shares_to_tokens(val.stake))?;

    stake.stake_validator(&mut val, stake_amount);
    STAKED.save(deps.storage, (&staker, &validator), &stake)?;
    VALIDATORS.save(deps.storage, &validator, &val)?;

    // set ack as data
    Ok(Response::new().set_data(StdAck::success("1")))
}

pub fn reply_unstake_callback(
    deps: DepsMut,
    data: Binary,
) -> Result<Response, ContractError> {
    let CallbackDataResponse {
        validator,
        staker,
        stake_amount,
        rewards,
    } = verify_callback_data(data)?;

    // We trust provider tested validator exists, so we can safely load default validator if we
    // haven't staked for this validator before.
    let mut val = VALIDATORS
        .may_load(deps.storage, &validator)?
        .unwrap_or_default();
    let mut stake = STAKED
        .may_load(deps.storage, (&staker, &validator))?
        .unwrap_or_default();

    // calculate validator rewards and save.
    val.calc_rewards(rewards.amount)?;

    // calculate rewards of delegator
    stake.calc_pending_rewards(val.rewards_per_token, val.shares_to_tokens(val.stake))?;

    stake.unstake_validator(&mut val, stake_amount)?;
    STAKED.save(deps.storage, (&staker, &validator), &stake)?;
    VALIDATORS.save(deps.storage, &validator, &val)?;

    // set ack as data
    Ok(Response::new().set_data(StdAck::success("1")))
}

pub fn reply_withdraw_rewards_callback(
    deps: DepsMut,
    env: Env,
    data: Binary,
) -> Result<Response, ContractError> {
    let CallbackDataResponse {
        validator,
        staker,
        stake_amount: _,
        rewards,
    } = verify_callback_data(data)?;
    let channel = CHANNEL.load(deps.storage)?;
    // We trust provider tested validator exists, so we can safely load default validator if we
    // haven't staked for this validator before.
    let mut val = VALIDATORS
        .may_load(deps.storage, &validator)?
        .unwrap_or_default();
    let mut stake = STAKED
        .may_load(deps.storage, (&staker, &validator))?
        .unwrap_or_default();

    // calculate validator rewards and save.
    val.calc_rewards(rewards.amount)?;

    // calculate rewards of delegator
    stake.calc_pending_rewards(val.rewards_per_token, val.shares_to_tokens(val.stake))?;

    let mut response = Response::new();

    if !stake.rewards.pending.floor().is_zero() {
        // if its not zero, we can send something, else just return an empty response, no error.
        let denom = deps.querier.query_bonded_denom()?;

        let consumer_balance = deps
            .querier
            .query_balance(env.contract.address.clone(), denom.clone())?;

        // Can we return error here and it will be acknoledged as ack::error?
        // Because reply response should be the last response read, so it should really be ibcResponse.
        if stake.rewards.pending.floor() > Decimal::new(consumer_balance.amount) {
            return Err(ContractError::WrongBalance {
                balance: Decimal::new(consumer_balance.amount),
                rewards: stake.rewards.pending,
            });
        }

        let send_amount = stake.pending_to_u128()?;

        // Send ibc coin directly to delegator
        let msg = IbcMsg::Transfer {
            to_address: staker.clone(),
            amount: coin(send_amount, denom),
            channel_id: channel,
            timeout: build_timeout(deps.as_ref(), &env)?,
        };

        // Save new rewards
        stake.reset_pending();
        STAKED.save(deps.storage, (&staker, &validator), &stake)?;

        response = response.add_message(msg);
    }

    // set success, if something errored above, it should send ack::error???
    Ok(response.set_data(StdAck::success("1")))
}

/// Only handle errors in send
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn ibc_packet_ack(
    _deps: DepsMut,
    _env: Env,
    msg: IbcPacketAckMsg,
) -> Result<IbcBasicResponse, ContractError> {
    let res: StdAck = from_slice(&msg.acknowledgement.data)?;
    if res.is_err() {
        return Err(ContractError::AckFailed {});
    }

    // We need to parse the ack based on our request
    let original_packet: ConsumerMsg = from_slice(&msg.original_packet.data)?;
    match original_packet {
        ConsumerMsg::UpdateValidators {
            added: _,
            removed: _,
        } => Ok(IbcBasicResponse::new()),
    }
}

// The provder received our update packet, send the ics20 tokens.
// NOTE: This is required because ibcMsg::sendPacket can't we sent with other IbcMsgs in the same call.
pub fn acknowledge_rewards(
    deps: DepsMut,
    env: Env,
    amount: Coin,
) -> Result<IbcBasicResponse, ContractError> {
    let config = CONFIG.load(deps.storage)?;

    // NOTE We try to split the addr from the port_id, maybe better to set the addr in init?
    let provider_addr = config.provider.port_id.split('.').last();
    let provider_addr = match provider_addr {
        Some(addr) => addr,
        None => return Err(ContractError::ProviderAddrParsing {}),
    };

    let msg = IbcMsg::Transfer {
        channel_id: config.ics20_channel.clone(),
        to_address: provider_addr.to_string(),
        amount,
        timeout: build_timeout(deps.as_ref(), &env)?,
    };

    Ok(IbcBasicResponse::new().add_message(msg))
}

/// Handle timeout like ack errors
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn ibc_packet_timeout(
    deps: DepsMut,
    _env: Env,
    msg: IbcPacketTimeoutMsg,
) -> Result<IbcBasicResponse, ContractError> {
    // we need to parse the ack based on our request
    let original_packet: ConsumerMsg = from_slice(&msg.packet.data)?;
    match original_packet {
        ConsumerMsg::UpdateValidators { added, removed } => {
            fail_update_validators(deps, added, removed)
        }
    }
}

pub fn fail_rewards(_deps: DepsMut) -> Result<IbcBasicResponse, ContractError> {
    // TODO what should we do on failure to withdraw rewards?
    Err(ContractError::RewardsFailed {})
}

pub fn fail_update_validators(
    _deps: DepsMut,
    _added: Vec<String>,
    _removed: Vec<String>,
) -> Result<IbcBasicResponse, ContractError> {
    Err(ContractError::UpdateValidatorsFailed {})
}
