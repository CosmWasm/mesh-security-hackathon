#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;

use cosmwasm_std::{
    from_slice, to_binary, Coin, Deps, DepsMut, Env, Ibc3ChannelOpenResponse, IbcBasicResponse,
    IbcChannelCloseMsg, IbcChannelConnectMsg, IbcChannelOpenMsg, IbcMsg,
    IbcPacketReceiveMsg, IbcPacketTimeoutMsg, IbcReceiveResponse, IbcTimeout, Uint128, Event, StdError, IbcPacketAckMsg,
};

use mesh_ibc::{
    check_order, check_version, ConsumerMsg, ListValidatorsResponse, ProviderMsg, RewardsResponse,
    StdAck, UpdateValidatorsResponse,
};

use crate::error::ContractError;
use crate::state::{ValStatus, Validator, CHANNEL, CONFIG, PACKET_LIFETIME, PORT, VALIDATORS, STAKED, RETRIES};

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

    let channel = msg.channel();
    check_order(&channel.order)?;
    check_version(&channel.version)?;
    if let Some(counter_version) = msg.counterparty_version() {
        check_version(counter_version)?;
    }

    // ensure the remote connection / port is authorized
    let cfg = CONFIG.load(deps.storage)?;
    if cfg.consumer.connection_id != channel.connection_id {
        return Err(ContractError::WrongConnection(cfg.consumer.connection_id));
    }

    Ok(None)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn ibc_channel_connect(
    deps: DepsMut,
    env: Env,
    msg: IbcChannelConnectMsg,
) -> Result<IbcBasicResponse, ContractError> {
    let channel = msg.channel();
    let channel_id = &channel.endpoint.channel_id;
    let port_id = &channel.endpoint.port_id;

    // save the channel id for future use
    match CHANNEL.may_load(deps.storage)? {
        Some(chan) => return Err(ContractError::ChannelExists(chan)),
        None => CHANNEL.save(deps.storage, channel_id)?,
    };

    // save the port id for future use
    match PORT.may_load(deps.storage)? {
        Some(port) => return Err(ContractError::PortExists(port)),
        None => PORT.save(deps.storage, port_id)?,
    };

    let packet = ProviderMsg::ListValidators {};
    let msg = IbcMsg::SendPacket {
        channel_id: channel_id.to_string(),
        data: to_binary(&packet)?,
        timeout: build_timeout(deps.as_ref(), &env)?,
    };
    Ok(IbcBasicResponse::new().add_message(msg))
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

    let msg: ConsumerMsg = from_slice(&msg.packet.data)?;
    match msg {
        ConsumerMsg::Rewards {
            validator,
            total_funds,
        } => receive_rewards(deps, env, validator, total_funds),
        ConsumerMsg::UpdateValidators { added, removed } => {
            receive_update_validators(deps, env, added, removed)
        }
    }
}

pub fn receive_rewards(
    deps: DepsMut,
    _env: Env,
    validator: String,
    total_funds: Coin,
) -> Result<IbcReceiveResponse, ContractError> {
    // Update the rewards of this validator
    // This will fail if we didn't add the validator before it, we cannot init the validator and calculate rewards in the same msg. (same block)
    VALIDATORS.update::<_, ContractError>(deps.storage, &validator, |val| {
        let mut val = match val {
            Some(val) => val,
            None => return Err(ContractError::ValidatorRewardsCalculationWrong {}),
        };

        let total_staked = val.shares_to_tokens(val.stake);

        if total_staked.is_zero() {
            return Err(ContractError::ValidatorRewardsCalculationWrong {});
        }

        val.rewards
            .calc_rewards(total_funds.amount, val.shares_to_tokens(val.stake))?;

        Ok(val)
    })?;

    // TODO: if calculation failed, we want to handle it as leftover funds? or send funds back to consumer and handle it there?
    let ack = StdAck::success(&RewardsResponse {});

    Ok(IbcReceiveResponse::new().set_ack(ack))
}

pub fn receive_update_validators(
    deps: DepsMut,
    _env: Env,
    added: Vec<String>,
    removed: Vec<String>,
) -> Result<IbcReceiveResponse, ContractError> {
    for add in added {
        if !VALIDATORS.has(deps.storage, &add) {
            VALIDATORS.save(deps.storage, &add, &Validator::new())?;
        }
    }
    for remove in removed {
        if let Some(mut val) = VALIDATORS.may_load(deps.storage, &remove)? {
            val.status = ValStatus::Removed;
            VALIDATORS.save(deps.storage, &remove, &val)?;
        }
    }
    let ack = StdAck::success(&UpdateValidatorsResponse {});
    Ok(IbcReceiveResponse::new().set_ack(ack))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn ibc_packet_ack(
    deps: DepsMut,
    env: Env,
    msg: IbcPacketAckMsg,
) -> Result<IbcBasicResponse, ContractError> {
    let res: StdAck = from_slice(&msg.acknowledgement.data)?;
    // we need to handle the ack based on our request
    let original_packet: ProviderMsg = from_slice(&msg.original_packet.data)?;
    match (original_packet, res.is_ok()) {
        (ProviderMsg::ListValidators {}, true) => {
            let val: ListValidatorsResponse = from_slice(&res.unwrap())?;
            ack_list_validators(deps, env, val)
        }
        (ProviderMsg::ListValidators {}, false) => fail_list_validators(deps, env),
        (
            ProviderMsg::Stake {
                key,
                validator,
                amount,
            },
            true,
        ) => ack_stake(deps, key, validator, amount),
        (
            ProviderMsg::Stake {
                key,
                validator,
                amount,
            },
            false,
        ) => fail_stake(deps, env, key, validator, amount),
        (
            ProviderMsg::Unstake {
                key: _,
                validator: _,
                amount: _,
            },
            true,
        ) => ack_unstake(deps),
        (
            ProviderMsg::Unstake {
                key,
                validator,
                amount,
            },
            false,
        ) => fail_unstake(deps, env, key, validator, amount),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn ibc_packet_timeout(
    deps: DepsMut,
    env: Env,
    msg: IbcPacketTimeoutMsg,
) -> Result<IbcBasicResponse, ContractError> {
    let original_packet: ProviderMsg = from_slice(&msg.packet.data)?;
    match original_packet {
        ProviderMsg::ListValidators {} => fail_list_validators(deps, env),
        ProviderMsg::Stake {
            key,
            validator,
            amount,
        } => fail_stake(deps, env, key, validator, amount),
        ProviderMsg::Unstake {
            key,
            validator,
            amount,
        } => fail_unstake(deps, env, key, validator, amount),
    }
}

pub fn ack_list_validators(
    deps: DepsMut,
    _env: Env,
    res: ListValidatorsResponse,
) -> Result<IbcBasicResponse, ContractError> {
    for val in res.validators {
        VALIDATORS.save(deps.storage, &val, &Validator::new())?;
    }
    RETRIES.update(deps.storage, |mut r| -> Result<crate::state::RetryState, StdError> {
        r.list_validators_reset();
        Ok(r)
    })?;
    Ok(IbcBasicResponse::new()
        .add_attribute("action", "ack list_validators"))
}

pub fn fail_list_validators(deps: DepsMut, env: Env) -> Result<IbcBasicResponse, ContractError> {
    // check if we should retry
    let mut retries = RETRIES.load(deps.storage)?;
    if !retries.list_validators_should_retry() {
        retries.list_validators_reset();
        RETRIES.save(deps.storage, &retries)?;
        return Ok(IbcBasicResponse::new().add_event(Event::new("list_validators_fail")))
    }
    RETRIES.save(deps.storage, &retries)?;

    // retry
    let channel_id = CHANNEL.load(deps.storage)?;
    let packet = ProviderMsg::ListValidators {};
    let msg = IbcMsg::SendPacket {
        channel_id: channel_id,
        data: to_binary(&packet)?,
        timeout: build_timeout(deps.as_ref(), &env)?,
    };
    Ok(IbcBasicResponse::new()
        .add_event(Event::new("list_validators_retry"))
        .add_message(msg))
}

fn ack_stake(deps: DepsMut, staker: String, validator: String, amount: Uint128) -> Result<IbcBasicResponse, ContractError> {
    let staker = deps.api.addr_validate(&staker)?;

    if amount.is_zero() {
        return Ok(IbcBasicResponse::new().add_event(Event::new("ack_stake_zero_amount")));
    }

    let mut val = VALIDATORS
        .may_load(deps.storage, &validator)?
        .ok_or_else(|| ContractError::UnknownValidator(validator.clone()))?;
    let mut stake = STAKED.load(deps.storage, (&staker, &validator))?;
    stake.stake_validator(&mut val, amount);
    STAKED.save(deps.storage, (&staker, &validator), &stake)?;
    VALIDATORS.save(deps.storage, &validator, &val)?;

    Ok(IbcBasicResponse::new()
        .add_event(Event::new("ack_stake")))
}

pub fn fail_stake(
    deps: DepsMut,
    env: Env,
    staker: String,
    validator: String,
    amount: Uint128,
) -> Result<IbcBasicResponse, ContractError> {
    let mut retries = RETRIES.load(deps.storage)?;
    let packet = ProviderMsg::Stake {
        validator,
        amount,
        key: staker.into(),
    };
    let msg = IbcMsg::SendPacket {
        channel_id: CHANNEL.load(deps.storage)?,
        data: to_binary(&packet)?,
        timeout: build_timeout(deps.as_ref(), &env)?,
    };
    Ok(IbcBasicResponse::new()
        .add_event(Event::new("retry_stake"))
        .add_message(msg))
}

pub fn ack_unstake(
    deps: DepsMut,
) -> Result<IbcBasicResponse , ContractError> {
    Ok(IbcBasicResponse::new().add_event(Event::new("ack_unstake")))
}

pub fn fail_unstake(
    deps: DepsMut,
    env: Env,
    staker: String,
    validator: String,
    amount: Uint128,
) -> Result<IbcBasicResponse, ContractError> {
    let packet = ProviderMsg::Unstake {
        validator,
        amount,
        key: staker.to_string(),
    };
    let msg = IbcMsg::SendPacket {
        channel_id: CHANNEL.load(deps.storage)?,
        data: to_binary(&packet)?,
        timeout: build_timeout(deps.as_ref(), &env)?,
    };
    Ok(IbcBasicResponse::new()
        .add_event(Event::new("retry_unstake"))
        .add_message(msg))
}
