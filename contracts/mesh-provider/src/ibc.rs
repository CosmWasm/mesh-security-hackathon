use std::collections::HashMap;

#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;

use cosmwasm_std::{
    from_slice, to_binary, Addr, Coin, DepsMut, Env, Ibc3ChannelOpenResponse, IbcBasicResponse,
    IbcChannelCloseMsg, IbcChannelConnectMsg, IbcChannelOpenMsg, IbcMsg, IbcPacketAckMsg,
    IbcPacketReceiveMsg, IbcPacketTimeoutMsg, IbcReceiveResponse, IbcTimeout, Order, StdError,
    StdResult, Uint128,
};

use mesh_ibc::{
    check_order, check_version, ConsumerMsg, ListValidatorsResponse, ProviderMsg, StdAck,
    UpdateValidatorsResponse, RewardsResponse,
};

use crate::error::ContractError;
use crate::state::{
    Stake, ValStatus, Validator, CHANNEL, CONFIG, PORT, REWARDS, STAKED_BY_VALIDATOR, VALIDATORS,
};

// TODO: make configurable?
/// packets live one hour
const PACKET_LIFETIME: u64 = 60 * 60;

pub fn build_timeout(env: &Env) -> IbcTimeout {
    let time = env.block.time.plus_seconds(PACKET_LIFETIME);
    IbcTimeout::with_timestamp(time)
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
        None => CHANNEL.save(deps.storage, port_id)?,
    };

    let packet = ProviderMsg::ListValidators {};
    let msg = IbcMsg::SendPacket {
        channel_id: channel_id.to_string(),
        data: to_binary(&packet)?,
        timeout: build_timeout(&env),
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
    _env: Env,
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
            rewards_by_validator,
            denom,
        } => receive_rewards(deps, rewards_by_validator, denom),
        ConsumerMsg::UpdateValidators { added, removed } => {
            receive_update_validators(deps, added, removed)
        }
    }
}

pub fn receive_rewards(
    deps: DepsMut,
    rewards_by_validator: Vec<(String, Uint128)>,
    denom: String,
) -> Result<IbcReceiveResponse, ContractError> {
    let port_id = PORT.load(deps.storage)?;
    let channel_id = CHANNEL.load(deps.storage)?;
    let ibc_denom = format!("{}/{}/{}", &port_id, &channel_id, &denom);

    rewards_by_validator.iter().for_each(|res| {
        let (val, total_rewards_amount) = res;

        let total_shares_staked = VALIDATORS
            .load(deps.storage, &val)
            .unwrap_or_default()
            .stake;
        let staked_by_validator = STAKED_BY_VALIDATOR
            .prefix(&val)
            .range(deps.storage, None, None, Order::Ascending)
            .collect::<StdResult<Vec<(Addr, Stake)>>>()
            .unwrap_or_default();

        staked_by_validator.iter().for_each(|res| {
            let (delegator, stake) = res;

            let perc = (stake.shares / total_shares_staked).u128() * (100_u128);
            let final_amount = (perc * total_rewards_amount.u128()) / (100_u128);

            REWARDS
                .update::<_, StdError>(deps.storage, &delegator, |coins| match coins {
                    Some(mut coins) => {
                        let mut empty_coin = Coin {
                            denom: ibc_denom.to_string(),
                            amount: Uint128::from(0_u128),
                        };
                        let exists = coins.get_mut(&ibc_denom).unwrap_or(&mut empty_coin);

                        exists.amount += Uint128::from(final_amount);
                        Ok(coins)
                    }
                    None => {
                        let mut coins = HashMap::new();
                        coins.insert(
                            ibc_denom.to_string(),
                            Coin {
                                denom: ibc_denom.to_string(),
                                amount: Uint128::from(final_amount),
                            },
                        );
                        Ok(coins)
                    }
                })
                .unwrap();
        })
    });

    let ack = StdAck::success(&RewardsResponse {});
    Ok(IbcReceiveResponse::new().set_ack(ack))
}

pub fn receive_update_validators(
    deps: DepsMut,
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
    _env: Env,
    msg: IbcPacketAckMsg,
) -> Result<IbcBasicResponse, ContractError> {
    let res: StdAck = from_slice(&msg.acknowledgement.data)?;

    // TODO remove, but temporarily useful for debugging
    if res.is_err() {
        panic!("ack: {:?}", res.unwrap_err());
    }

    // we need to handle the ack based on our request
    let original_packet: ProviderMsg = from_slice(&msg.original_packet.data)?;
    match (original_packet, res.is_ok()) {
        (ProviderMsg::ListValidators {}, true) => {
            let val: ListValidatorsResponse = from_slice(&res.unwrap())?;
            ack_list_validators(deps, val)
        }
        (ProviderMsg::ListValidators {}, false) => fail_list_validators(deps),
        (
            ProviderMsg::Stake {
                key,
                validator,
                amount,
            },
            false,
        ) => fail_stake(deps, key, validator, amount),
        (
            ProviderMsg::Unstake {
                key,
                validator,
                amount,
            },
            false,
        ) => fail_unstake(deps, key, validator, amount),
        (_, true) => Ok(IbcBasicResponse::new()),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn ibc_packet_timeout(
    deps: DepsMut,
    _env: Env,
    msg: IbcPacketTimeoutMsg,
) -> Result<IbcBasicResponse, ContractError> {
    let original_packet: ProviderMsg = from_slice(&msg.packet.data)?;
    match original_packet {
        ProviderMsg::ListValidators {} => fail_list_validators(deps),
        ProviderMsg::Stake {
            key,
            validator,
            amount,
        } => fail_stake(deps, key, validator, amount),
        ProviderMsg::Unstake {
            key,
            validator,
            amount,
        } => fail_unstake(deps, key, validator, amount),
    }
}

pub fn ack_list_validators(
    deps: DepsMut,
    res: ListValidatorsResponse,
) -> Result<IbcBasicResponse, ContractError> {
    for val in res.validators {
        VALIDATORS.save(deps.storage, &val, &Validator::new())?;
    }
    Ok(IbcBasicResponse::new())
}

pub fn fail_list_validators(_deps: DepsMut) -> Result<IbcBasicResponse, ContractError> {
    // TODO: send another ListValidators message
    unimplemented!();
}

pub fn fail_stake(
    _deps: DepsMut,
    // _staker is the staker Addr, not used by consumer
    _staker: String,
    _validator: String,
    _amount: Uint128,
) -> Result<IbcBasicResponse, ContractError> {
    // TODO: release the bonded stake, adjust numer
    unimplemented!();
}

pub fn fail_unstake(
    _deps: DepsMut,
    // _staker is the staker Addr, not used by consumer
    _staker: String,
    _validator: String,
    _amount: Uint128,
) -> Result<IbcBasicResponse, ContractError> {
    // TODO: unrelease the bonded stake, remove claim
    // Maybe we only make Claim on ack?
    unimplemented!();
}
