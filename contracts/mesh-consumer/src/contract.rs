#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, Binary, Deps, DepsMut, Env, IbcMsg, IbcTimeout, MessageInfo, Response, StdResult,
};
use cw2::set_contract_version;

use mesh_apis::ConsumerExecuteMsg;
use mesh_ibc::ConsumerMsg;

use crate::error::ContractError;
use crate::msg::{InstantiateMsg, QueryMsg};
use crate::state::{Config, CHANNEL, CONFIG};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:mesh-consumer";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    let meta_staking_contract_address =
        deps.api.addr_validate(&msg.meta_staking_contract_address)?;

    let config = Config {
        meta_staking_contract_address,
        provider: msg.provider,
        remote_to_local_exchange_rate: msg.remote_to_local_exchange_rate,
        ics20_channel: msg.ics20_channel,
    };
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    CONFIG.save(deps.storage, &config)?;

    Ok(Response::new().add_attribute("method", "instantiate"))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ConsumerExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ConsumerExecuteMsg::MeshConsumerRecieveRewardsMsg {
            validator
        } => execute_receive_rewards(deps, env, info, validator),
    }
}

// We receive the rewards as funds from meta-stacking, and send it over IBC to mesh-provider
pub fn execute_receive_rewards(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    validator: String,
) -> Result<Response, ContractError> {
    let channel_id = CHANNEL.load(deps.storage)?;
    let timeout: IbcTimeout = env.block.time.plus_seconds(300).into();

    let coin = info.funds[0].clone();

    let msg = IbcMsg::SendPacket {
        channel_id,
        data: to_binary(&ConsumerMsg::Rewards {
            validator,
            total_funds: coin,
        })?,
        timeout,
    };

    Ok(Response::default().add_message(msg))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(_deps: Deps, _env: Env, _msg: QueryMsg) -> StdResult<Binary> {
    unimplemented!();
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::msg::ProviderInfo;
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    use cosmwasm_std::{coins, Decimal};

    fn provider_info() -> ProviderInfo {
        ProviderInfo {
            port_id: "port-1".to_string(),
            connection_id: "conn-2".to_string(),
        }
    }

    #[test]
    fn proper_initialization() {
        let mut deps = mock_dependencies();

        let msg = InstantiateMsg {
            meta_staking_contract_address: "meta_staking".to_string(),
            provider: provider_info(),
            remote_to_local_exchange_rate: Decimal::percent(10),
            ics20_channel: "channel-10".to_string(),
        };
        let info = mock_info("creator", &coins(1000, "earth"));

        // we can just call .unwrap() to assert this was a success
        let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(0, res.messages.len());
    }
}
