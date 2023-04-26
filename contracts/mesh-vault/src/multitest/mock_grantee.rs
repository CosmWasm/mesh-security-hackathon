use cosmwasm_schema::cw_serde;
use cosmwasm_std::{
    to_binary, Addr, Binary, Deps, DepsMut, Empty, Env, MessageInfo, Response, StdResult, Uint128,
    WasmMsg,
};

use cw_multi_test::{Contract, ContractWrapper};
use cw_storage_plus::Item;

pub fn contract_mock() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(execute, instantiate, query);
    Box::new(contract)
}

#[cw_serde]
pub struct InstantiateMsg {
    /// Address of Lockup contract from which we accept ReceiveClaim
    pub lockup: String,
}

#[cw_serde]
pub enum ExecuteMsg {
    /// This gives the receiver access to slash part up to this much claim
    ReceiveClaim {
        owner: String,
        amount: Uint128,
        validator: String,
    },
    /// This releases a previously received claim without slashing it
    Release { owner: String, amount: Uint128 },
    /// This slashes a previously provided claim
    Slash { owner: String, amount: Uint128 },
}

#[cw_serde]
pub enum QueryMsg {
    DoNothing {},
}

const LOCKUP: Item<Addr> = Item::new("lockup");

pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> StdResult<Response> {
    let addr = deps.api.addr_validate(&msg.lockup)?;
    LOCKUP.save(deps.storage, &addr)?;
    Ok(Response::new())
}

pub fn execute(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: ExecuteMsg,
) -> StdResult<Response> {
    match msg {
        ExecuteMsg::ReceiveClaim { .. } => Ok(Response::new()),
        ExecuteMsg::Release { owner, amount } => {
            let msg = WasmMsg::Execute {
                contract_addr: LOCKUP.load(deps.storage)?.into_string(),
                msg: to_binary(&crate::msg::ExecuteMsg::ReleaseClaim { owner, amount })?,
                funds: vec![],
            };
            Ok(Response::new().add_message(msg))
        }
        ExecuteMsg::Slash { owner, amount } => {
            let msg = WasmMsg::Execute {
                contract_addr: LOCKUP.load(deps.storage)?.into_string(),
                msg: to_binary(&crate::msg::ExecuteMsg::SlashClaim { owner, amount })?,
                funds: vec![],
            };
            Ok(Response::new().add_message(msg))
        }
    }
}

pub fn query(_deps: Deps, _env: Env, _msg: QueryMsg) -> StdResult<Binary> {
    unimplemented!()
}
