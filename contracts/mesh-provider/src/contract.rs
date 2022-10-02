#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    Binary, Deps, DepsMut, Env, MessageInfo, Reply, Response, StdResult, SubMsg, SubMsgResponse,
    WasmMsg,
};
use cw2::set_contract_version;
use cw_utils::parse_instantiate_response_data;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::{Config, CONFIG};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:mesh-provider";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

// for reply callbacks
const INIT_CALLBACK_ID: u64 = 1;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    let state = Config {
        consumer: msg.consumer,
        slasher: None,
    };
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    CONFIG.save(deps.storage, &state)?;

    let label = format!("Slasher for {}", &env.contract.address);
    let msg = WasmMsg::Instantiate {
        admin: Some(env.contract.address.into_string()),
        code_id: msg.slasher.code_id,
        msg: msg.slasher.msg,
        funds: vec![],
        label,
    };
    let msg = SubMsg::reply_on_success(msg, INIT_CALLBACK_ID);

    Ok(Response::new()
        .add_submessage(msg)
        .add_attribute("method", "instantiate")
        .add_attribute("owner", info.sender))
}

#[entry_point]
pub fn reply(deps: DepsMut, _env: Env, reply: Reply) -> Result<Response, ContractError> {
    match reply.id {
        INIT_CALLBACK_ID => reply_init_callback(deps, reply.result.unwrap()),
        _ => Err(ContractError::InvalidReplyId(reply.id)),
    }
}

pub fn reply_init_callback(deps: DepsMut, resp: SubMsgResponse) -> Result<Response, ContractError> {
    CONFIG.update::<_, ContractError>(deps.storage, |mut cfg| {
        let init_response = parse_instantiate_response_data(&resp.data.unwrap_or_default())?;
        cfg.slasher = Some(deps.api.addr_validate(&init_response.contract_address)?);
        Ok(cfg)
    })?;
    Ok(Response::new())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    _deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    unimplemented!();
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(_deps: Deps, _env: Env, _msg: QueryMsg) -> StdResult<Binary> {
    unimplemented!();
}

#[cfg(test)]
mod tests {
    use crate::msg::{ConsumerInfo, SlasherInfo};

    use super::*;
    use cosmwasm_std::coins;
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};

    #[test]
    fn proper_initialization() {
        let mut deps = mock_dependencies();

        let msg = InstantiateMsg {
            consumer: ConsumerInfo {
                connection_id: "1".to_string(),
            },
            slasher: SlasherInfo {
                code_id: 17,
                msg: b"{}".into(),
            },
        };
        let info = mock_info("creator", &coins(1000, "earth"));

        // we can just call .unwrap() to assert this was a success
        let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(1, res.messages.len());
    }
}
