use std::{
    collections::HashMap,
    fmt::{Debug, Display},
    marker::PhantomData,
};

use anyhow::Error;
use cosmwasm_std::{
    coins,
    testing::{MockApi, MockStorage},
    Addr, Api, Coin, Empty, QuerierWrapper, StdResult, Storage,
};
use cw_multi_test::{
    next_block, App, AppBuilder, AppResponse, BankKeeper, BankSudo, Contract, DistributionKeeper,
    Executor, FailingModule, Router, StakeKeeper, SudoMsg, WasmKeeper, WasmSudo,
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

pub struct AppWrapper<E, IM, EM, QM, SM> {
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
    sudo_msg: PhantomData<SM>,
}

impl<E, IM, EM, QM, SM> AppWrapper<E, IM, EM, QM, SM> {
    /// Build the app
    pub fn build_app<F>(init_fn: F) -> AppWrapper<E, IM, EM, QM, SM>
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
            sudo_msg: PhantomData,
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

    /// Get contract Addr by name if provided (returns error if name no found)
    pub fn get_contract_addr(&self, name: &str) -> Result<&Addr, &str> {
        self.contracts.get(name).ok_or("Name doesn't exists")
    }

    // Get the module querier to query modules
    pub fn module_querier<'a>(&'a self) -> QuerierWrapper<'a> {
        self.app.wrap()
    }

    // Go to next block
    pub fn next_block(&mut self) {
        self.app.update_block(next_block);
    }

    pub fn update_block_seconds(&mut self, addition: u64) {
        self.app
            .update_block(|block| block.time = block.time.plus_seconds(addition));
    }

    // Read from module, maybe we can add some functionltiy here later
    pub fn read_module<F, T>(&self, query_fn: F) -> T
    where
        F: FnOnce(
            &Router<
                BankKeeper,
                FailingModule<Empty, Empty, Empty>,
                WasmKeeper<Empty, Empty>,
                StakeKeeper,
                DistributionKeeper,
            >,
            &dyn Api,
            &dyn Storage,
        ) -> T,
    {
        self.app.read_module(query_fn)
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
    fn init_contract_fail<M: Serialize>(
        &mut self,
        contract: StoreContract<String, M>,
    ) -> ExecuteResult<Addr, E>;
}

impl<E, IM, EM, QM, SM> AppInit<E, IM> for AppWrapper<E, IM, EM, QM, SM> {
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
    fn init_contract_fail<M: Serialize>(
        &mut self,
        contract: StoreContract<String, M>,
    ) -> ExecuteResult<Addr, E> {
        let contract_code_id = self.app.store_code(contract.contract_data);

        ExecuteResult(
            self.app.instantiate_contract(
                contract_code_id,
                ADMIN.addr(),
                &contract.init_msg,
                &[],
                "a_contract".to_string(),
                Some(ADMIN.addr().to_string()),
            ),
            PhantomData,
        )
    }
}

pub trait AppExecute<E, EM> {
    fn execute<M: Into<EM> + Serialize + Debug>(
        &mut self,
        contract_addr: Addr,
        sender: Addr,
        msg: M,
    ) -> ExecuteResult<AppResponse, E>;
    fn execute_with_funds<M: Into<EM> + Serialize + Debug>(
        &mut self,
        contract_addr: Addr,
        sender: Addr,
        funds: Vec<Coin>,
        msg: M,
    ) -> ExecuteResult<AppResponse, E>;
    fn execute_admin<M: Into<EM> + Serialize + Debug>(
        &mut self,
        contract_addr: Addr,
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

impl<E, IM, EM, QM, SM> AppExecute<E, EM> for AppWrapper<E, IM, EM, QM, SM> {
    fn execute<M>(
        &mut self,
        contract_addr: Addr,
        sender: Addr,
        msg: M,
    ) -> ExecuteResult<AppResponse, E>
    where
        M: Into<EM> + Serialize + Debug,
    {
        ExecuteResult(
            self.app.execute_contract(sender, contract_addr, &msg, &[]),
            PhantomData,
        )
    }

    fn execute_with_funds<M: Into<EM> + Serialize + Debug>(
        &mut self,
        contract_addr: Addr,
        sender: Addr,
        funds: Vec<Coin>,
        msg: M,
    ) -> ExecuteResult<AppResponse, E> {
        ExecuteResult(
            self.app
                .execute_contract(sender, contract_addr, &msg, &funds),
            PhantomData,
        )
    }

    fn execute_admin<M: Into<EM> + Serialize + Debug>(
        &mut self,
        contract_addr: Addr,
        msg: M,
    ) -> ExecuteResult<AppResponse, E> {
        ExecuteResult(
            self.app
                .execute_contract(ADMIN.addr(), contract_addr, &msg, &[]),
            PhantomData,
        )
    }
}

pub trait AppSudo<SM> {
    fn sudo<M: Into<SudoMsg> + Serialize + Debug>(&mut self, msg: M) -> Result<AppResponse, Error>;
    fn sudo_contract<M: Into<SM> + Serialize + Debug>(
        &mut self,
        contract_addr: &Addr,
        msg: M,
    ) -> Result<AppResponse, Error>;
}

impl<E, IM, EM, QM, SM> AppSudo<SM> for AppWrapper<E, IM, EM, QM, SM> {
    fn sudo<M: Into<SudoMsg> + Serialize + Debug>(&mut self, msg: M) -> Result<AppResponse, Error> {
        self.app.sudo(msg.into())
    }
    ///
    fn sudo_contract<M: Into<SM> + Serialize + Debug>(
        &mut self,
        contract_addr: &Addr,
        msg: M,
    ) -> Result<AppResponse, Error> {
        self.app
            .sudo(SudoMsg::Wasm(WasmSudo::new(contract_addr, &msg).unwrap()))
    }
}

pub trait AppQuery<QM> {
    fn query_smart<M: Into<QM> + Serialize + Debug + Clone, T: for<'a> Deserialize<'a>>(
        &mut self,
        contract_addr: &str,
        msg: M,
    ) -> StdResult<T>;
}

impl<E, IM, EM, QM, SM> AppQuery<QM> for AppWrapper<E, IM, EM, QM, SM> {
    fn query_smart<M, T>(&mut self, contract_addr: &str, msg: M) -> StdResult<T>
    where
        M: Into<QM> + Serialize + Debug + Clone,
        T: for<'a> Deserialize<'a>,
    {
        self.app.wrap().query_wasm_smart(contract_addr, &msg)
    }
}
