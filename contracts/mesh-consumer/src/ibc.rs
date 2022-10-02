#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;

use cosmwasm_std::{
    from_slice, DepsMut, Env, Ibc3ChannelOpenResponse, IbcBasicResponse, IbcChannelCloseMsg,
    IbcChannelConnectMsg, IbcChannelOpenMsg, IbcPacketAckMsg, IbcPacketReceiveMsg,
    IbcPacketTimeoutMsg, IbcReceiveResponse,
};

use mesh_ibc::{check_order, check_version, ConsumerMsg, ProviderMsg, StdAck, ValidatorAmount};

use crate::error::ContractError;
use crate::state::{CHANNEL, CONFIG};

// TODO: make configurable?
/// packets live one hour
pub const PACKET_LIFETIME: u64 = 60 * 60;

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
    _env: Env,
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
        ProviderMsg::Stake { validators, key: _ } => receive_stake(deps, validators),
        ProviderMsg::Unstake { validators, key: _ } => receive_unstake(deps, validators),
    }
}

pub fn receive_list_validators(_deps: DepsMut) -> Result<IbcReceiveResponse, ContractError> {
    // TODO
    unimplemented!();
}

pub fn receive_stake(
    _deps: DepsMut,
    _validators: Vec<ValidatorAmount>,
) -> Result<IbcReceiveResponse, ContractError> {
    // TODO
    unimplemented!();
}

pub fn receive_unstake(
    _deps: DepsMut,
    _validators: Vec<ValidatorAmount>,
) -> Result<IbcReceiveResponse, ContractError> {
    // TODO
    unimplemented!();
}

/// Only handle errors in send
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn ibc_packet_ack(
    deps: DepsMut,
    _env: Env,
    msg: IbcPacketAckMsg,
) -> Result<IbcBasicResponse, ContractError> {
    // we only care if there was an error...
    let res: StdAck = from_slice(&msg.acknowledgement.data)?;
    if res.is_ok() {
        return Ok(IbcBasicResponse::new());
    }

    // we need to parse the ack based on our request
    let original_packet: ConsumerMsg = from_slice(&msg.original_packet.data)?;
    match original_packet {
        ConsumerMsg::Rewards {} => fail_rewards(deps),
        ConsumerMsg::UpdateValidators { added, removed } => {
            fail_update_validators(deps, added, removed)
        }
    }
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
        ConsumerMsg::Rewards {} => fail_rewards(deps),
        ConsumerMsg::UpdateValidators { added, removed } => {
            fail_update_validators(deps, added, removed)
        }
    }
}

pub fn fail_rewards(_deps: DepsMut) -> Result<IbcBasicResponse, ContractError> {
    // TODO
    unimplemented!();
}

pub fn fail_update_validators(
    _deps: DepsMut,
    _added: Vec<String>,
    _removed: Vec<String>,
) -> Result<IbcBasicResponse, ContractError> {
    // TODO
    unimplemented!();
}