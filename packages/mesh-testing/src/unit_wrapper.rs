use cosmwasm_std::{
    coins, from_binary,
    testing::{mock_dependencies, mock_env, mock_info, MockApi, MockQuerier},
    Addr, Binary, Coin, Deps, DepsMut, Empty, Env, MemoryStorage, MessageInfo, OwnedDeps, Response,
    StdError, StdResult,
};
use serde::Deserialize;

use super::{ADMIN, NATIVE_DENOM};

/// Helper struct to hold all entry point functions for unit tests
pub struct ContractEntryPoints<E, IM, EM, QM, SM> {
    pub instantiate: fn(deps: DepsMut, env: Env, info: MessageInfo, msg: IM) -> Result<Response, E>,
    pub execute: fn(deps: DepsMut, env: Env, info: MessageInfo, msg: EM) -> Result<Response, E>,
    pub query: fn(deps: Deps, env: Env, msg: QM) -> StdResult<Binary>,
    pub sudo: fn(deps: DepsMut, env: Env, msg: SM) -> Result<Response, E>,
}

pub trait UnitExecute<EM, E> {
    fn execute<M: Into<EM>>(&mut self, sender: &str, msg: M) -> Result<Response, E>;
    fn execute_with_funds<M: Into<EM>>(&mut self, info: MessageInfo, msg: M)
        -> Result<Response, E>;
    fn execute_admin<M: Into<EM>>(&mut self, msg: M) -> Result<Response, E>;
}

pub trait UnitSudo<SM, E> {
    fn sudo<M: Into<SM>>(&mut self, msg: M) -> Result<Response, E>;
}

pub struct UnitQueryResult(pub Result<Binary, StdError>);

impl UnitQueryResult {
    pub fn unwrap<T: for<'a> Deserialize<'a>>(self) -> T {
        from_binary::<T>(&self.0.unwrap()).unwrap()
    }

    pub fn unwrap_err(self) -> StdError {
        self.0.unwrap_err()
    }
}

pub trait UnitQuery<QM> {
    fn query<M: Into<QM>>(&mut self, msg: M) -> UnitQueryResult;
}

pub struct ContractWrapper<E, IM, EM, QM, SM = Empty> {
    pub deps: OwnedDeps<MemoryStorage, MockApi, MockQuerier>,
    pub addr: Addr,
    pub contract_entry_points: ContractEntryPoints<E, IM, EM, QM, SM>,
}

impl<E: std::fmt::Debug, IM, EM, QM, SM> ContractWrapper<E, IM, EM, QM, SM> {
    pub fn init(
        contract_entry_points: ContractEntryPoints<E, IM, EM, QM, SM>,
        init_msg: IM,
    ) -> ContractWrapper<E, IM, EM, QM, SM> {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let init_info = mock_info(ADMIN.addr().as_str(), &coins(10000, NATIVE_DENOM));

        (contract_entry_points.instantiate)(
            deps.as_mut(),
            env.clone(),
            init_info.clone(),
            init_msg,
        )
        .unwrap();

        ContractWrapper {
            deps,
            addr: env.contract.address.clone(),
            contract_entry_points,
        }
    }

    /// Fund the contract with coins
    pub fn fund_contract(&mut self, coins: Vec<Coin>) {
        self.deps.querier.update_balance(self.addr.clone(), coins);
    }
}

impl<E, IM, EM, QM, SM> UnitExecute<EM, E> for ContractWrapper<E, IM, EM, QM, SM> {
    fn execute<M: Into<EM>>(&mut self, sender: &str, msg: M) -> Result<Response, E> {
        let info = mock_info(sender, &[]);
        (self.contract_entry_points.execute)(self.deps.as_mut(), mock_env(), info, msg.into())
    }

    fn execute_with_funds<M: Into<EM>>(
        &mut self,
        info: MessageInfo,
        msg: M,
    ) -> Result<Response, E> {
        (self.contract_entry_points.execute)(self.deps.as_mut(), mock_env(), info, msg.into())
    }

    fn execute_admin<M: Into<EM>>(&mut self, msg: M) -> Result<Response, E> {
        let info = mock_info(ADMIN.addr().as_str(), &[]);

        (self.contract_entry_points.execute)(self.deps.as_mut(), mock_env(), info, msg.into())
    }
}

impl<E, IM, EM, QM, SM> UnitSudo<SM, E> for ContractWrapper<E, IM, EM, QM, SM> {
    fn sudo<M: Into<SM>>(&mut self, msg: M) -> Result<Response, E> {
        (self.contract_entry_points.sudo)(self.deps.as_mut(), mock_env(), msg.into())
    }
}

impl<E, IM, EM, QM, SM> UnitQuery<QM> for ContractWrapper<E, IM, EM, QM, SM> {
    fn query<M: Into<QM>>(&mut self, msg: M) -> UnitQueryResult {
        UnitQueryResult((self.contract_entry_points.query)(
            self.deps.as_ref(),
            mock_env(),
            msg.into(),
        ))
    }
}
