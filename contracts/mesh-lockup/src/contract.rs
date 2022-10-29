#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, BankMsg, Binary, Coin, Deps, DepsMut, Env, MessageInfo, Response, StdResult,
    Uint128, WasmMsg,
};
use cw2::set_contract_version;
use cw_utils::{must_pay, nonpayable};

use crate::error::ContractError;
use crate::msg::{BalanceResponse, ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::{Config, BALANCES, CONFIG};
use mesh_apis::ClaimReceiverMsg;

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:mesh-lockup";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let cfg = Config { denom: msg.denom };
    CONFIG.save(deps.storage, &cfg)?;

    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("owner", info.sender))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Bond {} => execute_bond(deps, info),
        ExecuteMsg::Unbond { amount } => execute_unbond(deps, info, amount),
        ExecuteMsg::GrantClaim {
            provider,
            amount,
            validator,
        } => execute_grant_claim(deps, info, provider, amount, validator),
        ExecuteMsg::ReleaseClaim { owner, amount } => {
            execute_release_claim(deps, info, owner, amount)
        }
        ExecuteMsg::SlashClaim { owner, amount } => execute_slash_claim(deps, info, owner, amount),
    }
}

pub fn execute_bond(deps: DepsMut, info: MessageInfo) -> Result<Response, ContractError> {
    let denom = CONFIG.load(deps.storage)?.denom;
    let amount = must_pay(&info, &denom)?;

    BALANCES.update::<_, ContractError>(deps.storage, &info.sender, |old| {
        let mut old = old.unwrap_or_default();
        old.bonded += amount;
        Ok(old)
    })?;

    Ok(Response::new())
}

pub fn execute_unbond(
    deps: DepsMut,
    info: MessageInfo,
    amount: Uint128,
) -> Result<Response, ContractError> {
    nonpayable(&info)?;

    BALANCES.update::<_, ContractError>(deps.storage, &info.sender, |old| {
        // if they have nothing, we error (can we make it cleaner??)
        let mut acct = old.unwrap();
        let free = acct.free();
        if free < amount {
            return Err(ContractError::ClaimsLocked(free));
        }
        acct.bonded -= amount;
        Ok(acct)
    })?;

    let denom = CONFIG.load(deps.storage)?.denom;

    let msg = BankMsg::Send {
        to_address: info.sender.to_string(),
        amount: vec![Coin { denom, amount }],
    };

    Ok(Response::new().add_message(msg))
}

// this is called by the account holder
pub fn execute_grant_claim(
    deps: DepsMut,
    info: MessageInfo,
    provider: String,
    amount: Uint128,
    validator: String,
) -> Result<Response, ContractError> {
    nonpayable(&info)?;
    // validation
    let leiner = deps.api.addr_validate(&provider)?;

    // ensure we have balance for this, and update count
    BALANCES.update::<_, ContractError>(deps.storage, &info.sender, |bal| {
        let mut bal = bal.unwrap_or_default();
        bal.add_claim(&leiner, amount)?;
        Ok(bal)
    })?;

    // send a message to the receiver
    let exec = ClaimReceiverMsg::ReceiveClaim {
        owner: info.sender.into_string(),
        amount,
        validator,
    };
    let msg = WasmMsg::Execute {
        contract_addr: provider,
        msg: to_binary(&exec)?,
        funds: vec![],
    };

    Ok(Response::new().add_message(msg))
}

// this is called by the provider
pub fn execute_release_claim(
    deps: DepsMut,
    info: MessageInfo,
    owner: String,
    amount: Uint128,
) -> Result<Response, ContractError> {
    nonpayable(&info)?;
    let owner = deps.api.addr_validate(&owner)?;

    // ensure we have balance for this, and update count
    BALANCES.update::<_, ContractError>(deps.storage, &owner, |bal| {
        let mut bal = bal.unwrap_or_default();
        bal.release_claim(&info.sender, amount)?;
        Ok(bal)
    })?;

    Ok(Response::new())
}

pub fn execute_slash_claim(
    deps: DepsMut,
    info: MessageInfo,
    owner: String,
    amount: Uint128,
) -> Result<Response, ContractError> {
    nonpayable(&info)?;
    let owner = deps.api.addr_validate(&owner)?;

    // ensure we have balance for this, and update count
    BALANCES.update::<_, ContractError>(deps.storage, &owner, |bal| {
        let mut bal = bal.unwrap_or_default();
        bal.slash_claim(&info.sender, amount)?;
        Ok(bal)
    })?;

    Ok(Response::new())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Balance { account } => to_binary(&query_balance(deps, account)?),
    }
}

pub fn query_balance(deps: Deps, account: String) -> StdResult<BalanceResponse> {
    let account = deps.api.addr_validate(&account)?;
    let bal = BALANCES.load(deps.storage, &account)?;
    let free = bal.free();
    Ok(BalanceResponse {
        bonded: bal.bonded,
        free,
        claims: bal.claims.into_iter().map(Into::into).collect(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::coins;
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};

    const DENOM: &str = "uosmo";

    #[test]
    fn proper_initialization() {
        let mut deps = mock_dependencies();

        let msg = InstantiateMsg {
            denom: DENOM.to_string(),
        };
        let info = mock_info("creator", &coins(1000, "earth"));

        // we can just call .unwrap() to assert this was a success
        let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(0, res.messages.len());
    }
}
