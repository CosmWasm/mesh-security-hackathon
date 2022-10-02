use anyhow::Result as AnyResult;
use derivative::Derivative;

use cosmwasm_std::{coin, coins, Addr, Empty, StdResult, Uint128};
use cw_multi_test::{App, AppBuilder, AppResponse, Contract, ContractWrapper, Executor};

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
        let contract = app
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

impl Suite {
    pub fn bond(&mut self, executor: &str, amount: u128) -> AnyResult<AppResponse> {
        let funds = coins(amount, &self.denom);
        self.app.execute_contract(
            Addr::unchecked(executor),
            self.contract.clone(),
            &ExecuteMsg::Bond {},
            &funds,
        )
    }

    pub fn unbond(&mut self, executor: &str, amount: u128) -> AnyResult<AppResponse> {
        self.app.execute_contract(
            Addr::unchecked(executor),
            self.contract.clone(),
            &ExecuteMsg::Unbond {
                amount: amount.into(),
            },
            &[],
        )
    }

    //     /// This gives a claim on my balance to leinholder, granting it to a given validator
    //     GrantClaim {
    //     leinholder: String,
    //     amount: Uint128,
    //     validator: String,
    // },
    // /// This releases a previously received claim without slashing it
    // ReleaseClaim { owner: String, amount: Uint128 },
    // /// This slashes a previously provided claim
    // SlashClaim { owner: String, amount: Uint128 },

    pub fn ilp_balance(&self, account: impl Into<String>) -> StdResult<BalanceResponse> {
        self.app.wrap().query_wasm_smart(
            self.contract.clone(),
            &QueryMsg::Balance {
                account: account.into(),
            },
        )
    }

    pub fn balance(&self, account: impl Into<String>) -> StdResult<Uint128> {
        Ok(self.app.wrap().query_balance(account, &self.denom)?.amount)
    }
}
