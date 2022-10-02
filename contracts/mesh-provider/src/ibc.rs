#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;

use cosmwasm_std::{
    from_slice, DepsMut, Env, Ibc3ChannelOpenResponse, IbcBasicResponse, IbcChannelCloseMsg,
    IbcChannelConnectMsg, IbcChannelOpenMsg, IbcPacketAckMsg, IbcPacketReceiveMsg,
    IbcPacketTimeoutMsg, IbcReceiveResponse, StdResult,
};

use mesh_ibc::{
    check_order, check_version, ConsumerMsg, ListValidatorsResponse, ProviderMsg, StdAck,
};

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

    let msg: ConsumerMsg = from_slice(&msg.packet.data)?;
    match msg {
        ConsumerMsg::Rewards {} => receive_rewards(deps),
        ConsumerMsg::UpdateValidators { added, removed } => {
            receive_update_validators(deps, added, removed)
        }
    }
}

pub fn receive_rewards(_deps: DepsMut) -> Result<IbcReceiveResponse, ContractError> {
    // TODO
    unimplemented!();
}

pub fn receive_update_validators(
    _deps: DepsMut,
    _added: Vec<String>,
    _removed: Vec<String>,
) -> Result<IbcReceiveResponse, ContractError> {
    // TODO
    unimplemented!();
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn ibc_packet_ack(
    deps: DepsMut,
    _env: Env,
    msg: IbcPacketAckMsg,
) -> Result<IbcBasicResponse, ContractError> {
    let res: StdAck = from_slice(&msg.acknowledgement.data)?;
    if res.is_ok() {
        return Ok(IbcBasicResponse::new());
    }

    // we need to handle the ack based on our request
    let original_packet: ProviderMsg = from_slice(&msg.original_packet.data)?;
    match (original_packet, res.is_ok()) {
        (ProviderMsg::ListValidators {}, true) => {
            let res: ListValidatorsResponse = from_slice(&msg.acknowledgement.data)?;
            ack_list_validators(deps, res)
        }
        (ProviderMsg::ListValidators {}, false) => fail_list_validators(deps),
        (ProviderMsg::Stake { key, validators: _ }, _) => fail_stake(deps, key),
        (ProviderMsg::Unstake { key, validators: _ }, _) => fail_stake(deps, key),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
/// we just ignore these now. shall we store some info?
pub fn ibc_packet_timeout(
    _deps: DepsMut,
    _env: Env,
    _msg: IbcPacketTimeoutMsg,
) -> StdResult<IbcBasicResponse> {
    // TODO
    unimplemented!();
}

pub fn ack_list_validators(
    _deps: DepsMut,
    _res: ListValidatorsResponse,
) -> Result<IbcBasicResponse, ContractError> {
    // TODO
    unimplemented!();
}

pub fn fail_list_validators(_deps: DepsMut) -> Result<IbcBasicResponse, ContractError> {
    // TODO
    unimplemented!();
}

pub fn fail_stake(_deps: DepsMut, _key: u64) -> Result<IbcBasicResponse, ContractError> {
    // TODO
    unimplemented!();
}

pub fn fail_unstake(_deps: DepsMut, _key: u64) -> Result<IbcBasicResponse, ContractError> {
    // TODO
    unimplemented!();
}
