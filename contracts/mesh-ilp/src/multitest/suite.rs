use derivative::Derivative;

use cosmwasm_std::{coin, Addr, Empty};
use cw_multi_test::{App, AppBuilder, AppResponse, Contract, ContractWrapper, Executor};


pub fn contract_ilp() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(
        crate::contract::execute,
        crate::contract::instantiate,
        crate::contract::query,
    );
    Box::new(contract)
}

#[derive(Derivative)]
#[derivative(Default = "new")]
pub struct SuiteBuilder {
    funds: Vec<(Addr, u128)>,
    #[derivative(Default(value = "\"uosmo\".to_owned()"))]
    denom: String,
}

impl SuiteBuilder {
    /// Sets initial amount of distributable tokens on address
    pub fn with_funds(mut self, addr: impl Into<String>, amount: u128) -> Self {
        self.funds.push((Addr::unchecked(addr), amount));
        self
    }

    pub fn with_denom(mut self, denom: impl Into<String>) -> Self {
        self.denom = denom.into();
        self
    }

    #[track_caller]
    pub fn build(self) -> Suite {
        let denom = self.denom;
        let funds: Vec<_> = self
            .funds
            .iter()
            .map(|(addr, amount)| (addr, coin(*amount, &denom)))
            .collect();

        let mut app = AppBuilder::new().build(|router, _, storage| {
            for (addr, fund) in funds.into_iter() {
                router
                    .bank
                    .init_balance(storage, &addr, vec![fund])
                    .unwrap();
            }
        });

        let owner = Addr::unchecked("foobar");
        let contract_id = app.store_code(contract_ilp());
        let contract = app
            .instantiate_contract(
                contract_id,
                owner.clone(),
                &crate::msg::InstantiateMsg {
                    denom: denom.clone(),
                },
                &[],
                "ilp demo",
                None,
            )
            .unwrap();

        Suite {
            app,
            contract,
            owner,
            denom,
        }
    }
}

#[derive(Derivative)]
#[derivative(Debug)]
pub struct Suite {
    #[derivative(Debug = "ignore")]
    pub app: App,
    /// Engagement contract address
    pub contract: Addr,
    /// Mixer contract address
    pub owner: Addr,
    /// Denom of tokens which might be distributed by this contract
    pub denom: String,
}
