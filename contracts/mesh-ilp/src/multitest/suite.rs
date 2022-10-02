use anyhow::Result as AnyResult;
use derivative::Derivative;

use cosmwasm_std::{coin, coins, Addr, Empty, StdResult, Uint128};
use cw_multi_test::{App, AppBuilder, AppResponse, Contract, ContractWrapper, Executor};

use super::mock_grantee::contract_mock;
use crate::msg::{BalanceResponse, ExecuteMsg, InstantiateMsg, QueryMsg};

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
        let ilp_contract = app
            .instantiate_contract(
                contract_id,
                owner.clone(),
                &InstantiateMsg {
                    denom: denom.clone(),
                },
                &[],
                "ilp demo",
                None,
            )
            .unwrap();

        let mock_contract_id = app.store_code(contract_mock());
        let mock_contract = app
            .instantiate_contract(
                mock_contract_id,
                owner.clone(),
                &super::mock_grantee::InstantiateMsg {
                    ilp: ilp_contract.to_string(),
                },
                &[],
                "mock grantee",
                None,
            )
            .unwrap();

        Suite {
            app,
            ilp_contract,
            mock_contract,
            denom,
        }
    }
}

#[derive(Derivative)]
#[derivative(Debug)]
pub struct Suite {
    #[derivative(Debug = "ignore")]
    pub app: App,
    /// ILP contract address
    pub ilp_contract: Addr,
    /// Mock receiver address
    pub mock_contract: Addr,
    /// Denom of tokens which might be distributed by this contract
    pub denom: String,
}

impl Suite {
    pub fn bond(&mut self, executor: &str, amount: u128) -> AnyResult<AppResponse> {
        let funds = coins(amount, &self.denom);
        self.app.execute_contract(
            Addr::unchecked(executor),
            self.ilp_contract.clone(),
            &ExecuteMsg::Bond {},
            &funds,
        )
    }

    pub fn unbond(&mut self, executor: &str, amount: u128) -> AnyResult<AppResponse> {
        self.app.execute_contract(
            Addr::unchecked(executor),
            self.ilp_contract.clone(),
            &ExecuteMsg::Unbond {
                amount: amount.into(),
            },
            &[],
        )
    }

    pub fn grant_claim(
        &mut self,
        executor: &str,
        amount: u128,
        validator: &str,
    ) -> AnyResult<AppResponse> {
        self.app.execute_contract(
            Addr::unchecked(executor),
            self.ilp_contract.clone(),
            &ExecuteMsg::GrantClaim {
                leinholder: self.mock_contract.to_string(),
                amount: amount.into(),
                validator: validator.to_string(),
            },
            &[],
        )
    }

    pub fn release_claim(&mut self, executor: &str, amount: u128) -> AnyResult<AppResponse> {
        self.app.execute_contract(
            Addr::unchecked(executor),
            self.mock_contract.clone(),
            &ExecuteMsg::ReleaseClaim {
                owner: executor.to_string(),
                amount: amount.into(),
            },
            &[],
        )
    }

    pub fn slash_claim(&mut self, executor: &str, amount: u128) -> AnyResult<AppResponse> {
        self.app.execute_contract(
            Addr::unchecked(executor),
            self.mock_contract.clone(),
            &ExecuteMsg::SlashClaim {
                owner: executor.to_string(),
                amount: amount.into(),
            },
            &[],
        )
    }

    pub fn ilp_balance(&self, account: impl Into<String>) -> StdResult<BalanceResponse> {
        self.app.wrap().query_wasm_smart(
            self.ilp_contract.clone(),
            &QueryMsg::Balance {
                account: account.into(),
            },
        )
    }

    pub fn balance(&self, account: impl Into<String>) -> StdResult<Uint128> {
        Ok(self.app.wrap().query_balance(account, &self.denom)?.amount)
    }
}
