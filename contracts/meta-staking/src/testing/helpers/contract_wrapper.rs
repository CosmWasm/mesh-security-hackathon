use cosmwasm_std::{
    coins, from_binary,
    testing::{mock_env, mock_info, MockApi, MockQuerier},
    Addr, Binary, Coin, MemoryStorage, MessageInfo, OwnedDeps, Response, StdError,
};
use serde::Deserialize;

use crate::{
    contract::{execute, instantiate, query, sudo},
    error::ContractError,
    msg::{ExecuteMsg, InstantiateMsg, QueryMsg, SudoMsg},
};

use super::{ADMIN, NATIVE_DENOM};

pub trait Execute {
    fn execute<M: Into<ExecuteMsg>>(
        &mut self,
        sender: &str,
        msg: M,
    ) -> Result<Response, ContractError>;
    fn execute_with_funds<M: Into<ExecuteMsg>>(
        &mut self,
        info: MessageInfo,
        msg: M,
    ) -> Result<Response, ContractError>;
    fn execute_admin<M: Into<ExecuteMsg>>(&mut self, msg: M) -> Result<Response, ContractError>;
}

pub trait Sudo {
    fn sudo<M: Into<SudoMsg>>(&mut self, msg: M) -> Result<Response, ContractError>;
}

pub struct QueryResult(pub Result<Binary, StdError>);

impl QueryResult {
    pub fn unwrap<T: for<'a> Deserialize<'a>>(self) -> T {
        from_binary::<T>(&self.0.unwrap()).unwrap()
    }

    pub fn unwrap_err(self) -> StdError {
        self.0.unwrap_err()
    }
}

pub trait Query {
    fn query<M: Into<QueryMsg>>(&mut self, msg: M) -> QueryResult;
}

pub struct ContractWrapper {
    pub deps: OwnedDeps<MemoryStorage, MockApi, MockQuerier>,
    pub addr: Addr,
}

impl ContractWrapper {
    pub fn init(mut deps: OwnedDeps<MemoryStorage, MockApi, MockQuerier>) -> ContractWrapper {
        let init_info = mock_info(ADMIN.addr().as_str(), &coins(10000, NATIVE_DENOM));
        let env = mock_env();

        instantiate(
            deps.as_mut(),
            env.clone(),
            init_info.clone(),
            InstantiateMsg {},
        )
        .unwrap();

        ContractWrapper {
            deps,
            addr: env.contract.address.clone(),
        }
    }

    /// Fund the contract with coins
    pub fn fund_contract(&mut self, coins: Vec<Coin>) {
        self.deps.querier.update_balance(self.addr.clone(), coins);
    }
}

impl Execute for ContractWrapper {
    fn execute<M: Into<ExecuteMsg>>(
        &mut self,
        sender: &str,
        msg: M,
    ) -> Result<Response, ContractError> {
        let info = mock_info(sender, &[]);
        execute(self.deps.as_mut(), mock_env(), info, msg.into())
    }

    fn execute_with_funds<M: Into<ExecuteMsg>>(
        &mut self,
        info: MessageInfo,
        msg: M,
    ) -> Result<Response, ContractError> {
        execute(self.deps.as_mut(), mock_env(), info, msg.into())
    }

    fn execute_admin<M: Into<ExecuteMsg>>(&mut self, msg: M) -> Result<Response, ContractError> {
        let info = mock_info(ADMIN.addr().as_str(), &[]);

        execute(self.deps.as_mut(), mock_env(), info, msg.into())
    }
}

impl Sudo for ContractWrapper {
    fn sudo<M: Into<SudoMsg>>(&mut self, msg: M) -> Result<Response, ContractError> {
        sudo(self.deps.as_mut(), mock_env(), msg.into())
    }
}

impl Query for ContractWrapper {
    fn query<M: Into<QueryMsg>>(&mut self, msg: M) -> QueryResult {
        QueryResult(query(self.deps.as_ref(), mock_env(), msg.into()))
    }
}
