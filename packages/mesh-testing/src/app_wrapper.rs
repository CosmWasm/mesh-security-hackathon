use std::{
    collections::HashMap,
    fmt::{Debug, Display},
    marker::PhantomData,
};

use anyhow::Error;
use cosmwasm_std::{
    coins,
    testing::{MockApi, MockStorage},
    Addr, Api, Coin, Empty, StdResult, Storage,
};
use cw_multi_test::{
    App, AppBuilder, AppResponse, BankKeeper, BankSudo, Contract, DistributionKeeper, Executor,
    FailingModule, Router, StakeKeeper, SudoMsg, WasmKeeper,
};
use serde::{Deserialize, Serialize};

use super::{ADMIN, NATIVE_DENOM};

pub struct StoreContract<N, M> {
    name: Option<N>,
    contract_data: Box<dyn Contract<Empty>>,
    init_msg: M,
}

impl<N, M> StoreContract<N, M> {
    pub fn new(contract_data: Box<dyn Contract<Empty>>, init_msg: M) -> StoreContract<N, M> {
        StoreContract {
            contract_data,
            name: None,
            init_msg,
        }
    }

    pub fn new_with_name(
        name: N,
        contract_data: Box<dyn Contract<Empty>>,
        init_msg: M,
    ) -> StoreContract<N, M> {
        StoreContract {
            contract_data,
            name: Some(name),
            init_msg,
        }
    }
}

pub struct AppWrapper<E, IM, EM, QM> {
    pub app: App<
        BankKeeper,
        MockApi,
        MockStorage,
        FailingModule<Empty, Empty, Empty>,
        WasmKeeper<Empty, Empty>,
        StakeKeeper,
        DistributionKeeper,
    >,
    pub contracts: HashMap<String, Addr>,
    errors: PhantomData<E>,
    init_msg: PhantomData<IM>,
    execute_msg: PhantomData<EM>,
    query_msg: PhantomData<QM>,
}

impl<E, IM, EM, QM> AppWrapper<E, IM, EM, QM> {
    pub fn build_app<F>(init_fn: F) -> AppWrapper<E, IM, EM, QM>
    where
        F: FnOnce(
            &mut Router<
                BankKeeper,
                FailingModule<Empty, Empty, Empty>,
                WasmKeeper<Empty, Empty>,
                StakeKeeper,
                DistributionKeeper,
            >,
            &dyn Api,
            &mut dyn Storage,
        ),
    {
        let app = AppBuilder::new().build(init_fn);

        AppWrapper {
            app,
            contracts: HashMap::new(),
            errors: PhantomData,
            execute_msg: PhantomData,
            query_msg: PhantomData,
            init_msg: PhantomData,
        }
    }

    /// Fund address from sudo back
    pub fn fund_address(&mut self, address: Addr) {
        self.app
            .sudo(SudoMsg::Bank(BankSudo::Mint {
                to_address: address.to_string(),
                amount: coins(100000, NATIVE_DENOM),
            }))
            .unwrap();
    }

    // TODO: TO DELETE
    pub fn get_contract_addr(&self, name: &str) -> Result<&Addr, &str> {
        self.contracts.get(name).ok_or("Name doesn't exists")
    }
}

pub trait AppInit<E, IM> {
    fn init_contract<M: Serialize>(
        &mut self,
        sender: Addr,
        contract: StoreContract<&str, M>,
    ) -> Addr;
    fn init_contract_with_funds<M: Serialize>(
        &mut self,
        sender: Addr,
        contract: StoreContract<&str, M>,
        funds: Vec<Coin>,
    ) -> Addr;
    fn init_contract_fail<M: Serialize>(&mut self, contract: StoreContract<String, M>) -> ExecuteResult<Addr, E>;
}

impl<E, IM, EM, QM> AppInit<E, IM> for AppWrapper<E, IM, EM, QM> {
    fn init_contract<M: Serialize>(
        &mut self,
        sender: Addr,
        contract: StoreContract<&str, M>,
    ) -> Addr {
        let contract_code_id = self.app.store_code(contract.contract_data);
        let contract_name = match contract.name.clone() {
            Some(name) => name.into(),
            None => "a_contract".to_string(),
        };

        let contract_addr = self
            .app
            .instantiate_contract(
                contract_code_id,
                sender.clone(),
                &contract.init_msg,
                &[],
                contract_name,
                Some(sender.to_string()),
            )
            .unwrap();

        if let Some(name) = contract.name {
            self.contracts.insert(name.into(), contract_addr.clone());
        }
        contract_addr
    }

    /// Init contract with funds
    fn init_contract_with_funds<M: Serialize>(
        &mut self,
        sender: Addr,
        contract: StoreContract<&str, M>,
        funds: Vec<Coin>,
    ) -> Addr {
        let contract_code_id = self.app.store_code(contract.contract_data);
        let contract_name = match contract.name.clone() {
            Some(name) => name.clone().into(),
            None => "a_contract".to_string(),
        };

        let contract_addr = self
            .app
            .instantiate_contract(
                contract_code_id,
                sender.clone(),
                &contract.init_msg,
                &funds,
                contract_name,
                Some(sender.to_string()),
            )
            .unwrap();

        if let Some(name) = contract.name {
            self.contracts.insert(name.into(), contract_addr.clone());
        }
        contract_addr
    }

    /// Init a contract and expect it to fail
    /// use `.unwrap_err()` to get the Error and test it.
    fn init_contract_fail<M: Serialize>(&mut self, contract: StoreContract<String, M>) -> ExecuteResult<Addr, E>
    {
        let contract_code_id = self.app.store_code(contract.contract_data);

        ExecuteResult(self.app
            .instantiate_contract(
                contract_code_id,
                ADMIN.addr(),
                &contract.init_msg,
                &[],
                "a_contract".to_string(),
                Some(ADMIN.addr().to_string()),
            ), PhantomData)
    }
}

pub trait AppExecute<E, EM> {
    fn execute<M: Into<EM> + Serialize + Debug>(
        &mut self,
        contract_addr: &str,
        sender: Addr,
        msg: M,
    ) -> ExecuteResult<AppResponse, E>;
    fn execute_with_funds<M: Into<EM> + Serialize + Debug>(
        &mut self,
        contract_addr: &str,
        sender: Addr,
        funds: Vec<Coin>,
        msg: M,
    ) -> ExecuteResult<AppResponse, E>;
    fn execute_admin<M: Into<EM> + Serialize + Debug>(
        &mut self,
        contract_addr: &str,
        msg: M,
    ) -> ExecuteResult<AppResponse, E>;
}

pub struct ExecuteResult<T, E>(pub Result<T, Error>, PhantomData<E>);

impl<T: std::fmt::Debug, E: Display + Debug + Send + Sync + 'static> ExecuteResult<T, E> {
    pub fn unwrap(self) -> T {
        self.0.unwrap()
    }

    pub fn unwrap_err(self) -> E {
        self.0.unwrap_err().downcast().unwrap()
    }
}

impl<E, IM, EM, QM> AppExecute<E, EM> for AppWrapper<E, IM, EM, QM> {
    fn execute<M>(&mut self, contract_addr: &str, sender: Addr, msg: M) -> ExecuteResult<AppResponse, E>
    where
        M: Into<EM> + Serialize + Debug,
    {
        let contract_addr = self.get_contract_addr(contract_addr).unwrap().clone();
        ExecuteResult(
            self.app.execute_contract(sender, contract_addr, &msg, &[]),
            PhantomData,
        )
    }

    fn execute_with_funds<M: Into<EM> + Serialize + Debug>(
        &mut self,
        contract_addr: &str,
        sender: Addr,
        funds: Vec<Coin>,
        msg: M,
    ) -> ExecuteResult<AppResponse, E> {
        let contract_addr = self.get_contract_addr(contract_addr).unwrap().clone();
        ExecuteResult(
            self.app
                .execute_contract(sender, contract_addr, &msg, &funds),
            PhantomData,
        )
    }

    fn execute_admin<M: Into<EM> + Serialize + Debug>(
        &mut self,
        contract_addr: &str,
        msg: M,
    ) -> ExecuteResult<AppResponse, E> {
        let contract_addr = self.get_contract_addr(contract_addr).unwrap().clone();
        ExecuteResult(
            self.app
                .execute_contract(ADMIN.addr(), contract_addr, &msg, &[]),
            PhantomData,
        )
    }
}

pub trait AppSudo {
    fn sudo<M: Into<SudoMsg> + Serialize + Debug>(&mut self, msg: M) -> Result<AppResponse, Error>;
    fn sudo_contract<M: Into<SudoMsg> + Serialize + Debug>(
        &mut self,
        msg: M,
    ) -> Result<AppResponse, Error>;
}

impl<E, IM, EM, QM> AppSudo for AppWrapper<E, IM, EM, QM> {
    fn sudo<M: Into<SudoMsg> + Serialize + Debug>(&mut self, msg: M) -> Result<AppResponse, Error> {
        self.app.sudo(msg.into())
    }
    fn sudo_contract<M: Into<SudoMsg> + Serialize + Debug>(
        &mut self,
        msg: M,
    ) -> Result<AppResponse, Error> {
        self.app.sudo(msg.into())
    }
}

pub trait AppQuery<QM> {
    fn query_smart<M: Into<QM> + Serialize + Debug + Clone, T: for<'a> Deserialize<'a>>(
        &mut self,
        contract_addr: &str,
        msg: M,
    ) -> StdResult<T>;
}

impl<E, IM, EM, QM> AppQuery<QM> for AppWrapper<E, IM, EM, QM> {
    fn query_smart<M, T>(&mut self, contract_addr: &str, msg: M) -> StdResult<T>
    where
        M: Into<QM> + Serialize + Debug + Clone,
        T: for<'a> Deserialize<'a>,
    {
        let contract_addr = self.get_contract_addr(contract_addr).unwrap().clone();

        self.app.wrap().query_wasm_smart(contract_addr, &msg)
    }
}
