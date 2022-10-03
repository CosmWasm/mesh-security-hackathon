#[cfg(test)]
mod tests {
    use crate::msg::InstantiateMsg;
    use crate::{helpers::MetaStakingContract, msg::SudoMsg as MetaStakingSudoMsg};
    use cosmwasm_std::{to_binary, Addr, Coin, Decimal, Empty, Uint128};
    use cw_multi_test::{App, AppBuilder, Contract, ContractWrapper, Executor, SudoMsg, WasmSudo};

    pub fn meta_staking_contract() -> Box<dyn Contract<Empty>> {
        let contract = ContractWrapper::new(
            crate::contract::execute,
            crate::contract::instantiate,
            crate::contract::query,
        )
        .with_sudo(crate::contract::sudo);
        Box::new(contract)
    }

    const USER: &str = "USER";
    const ADMIN: &str = "ADMIN";
    const NATIVE_DENOM: &str = "denom";

    fn mock_app() -> App {
        AppBuilder::new().build(|router, _, storage| {
            router
                .bank
                .init_balance(
                    storage,
                    &Addr::unchecked(USER),
                    vec![Coin {
                        denom: NATIVE_DENOM.to_string(),
                        amount: Uint128::new(1),
                    }],
                )
                .unwrap();
        })
    }

    fn proper_instantiate() -> (App, MetaStakingContract) {
        let mut app = mock_app();
        let cw_template_id = app.store_code(meta_staking_contract());

        let msg = InstantiateMsg {
            local_denom: "ujuno".to_string(),
            provider_denom: "uosmo".to_string(),
            consumer_provider_exchange_rate: Decimal::percent(10),
        };
        let cw_template_contract_addr = app
            .instantiate_contract(
                cw_template_id,
                Addr::unchecked(ADMIN),
                &msg,
                &[],
                "test",
                None,
            )
            .unwrap();

        let meta_staking_addr = MetaStakingContract(cw_template_contract_addr);

        (app, meta_staking_addr)
    }

    #[test]
    fn happy_path() {
        let (mut app, meta_staking_addr) = proper_instantiate();

        let consumer = Addr::unchecked("consumer_contract");

        // Add consumer
        app.sudo(SudoMsg::Wasm(WasmSudo {
            contract_addr: consumer,
            msg: to_binary(&MetaStakingSudoMsg::UpdateConsumers {
                to_add: Some(vec![consumer.to_string()]),
                to_remove: None,
            })
            .unwrap(),
        }))
        .unwrap();

        // Fund contract successfully
        app.sudo(SudoMsg::Wasm(WasmSudo {
            contract_addr: meta_staking_addr.addr(),
            msg: to_binary(&MetaStakingSudoMsg::Fund {
                consumer: consumer.to_string(),
            })
            .unwrap(),
        }))
        .unwrap();
    }

    #[test]
    fn fail_fund_non_existent_consumer() {
        let (mut app, meta_staking_addr) = proper_instantiate();

        let consumer = Addr::unchecked("consumer_contract");

        // Funding wrong contract successfully
        app.sudo(SudoMsg::Wasm(WasmSudo {
            contract_addr: meta_staking_addr.addr(),
            msg: to_binary(&MetaStakingSudoMsg::Fund {
                consumer: consumer.to_string(),
            })
            .unwrap(),
        }))
        .unwrap_err();
    }

    #[test]
    fn only_gov_module_can_add_or_remove_consumers() {
        let (app, meta_staking_addr) = proper_instantiate();

        // Fund contract fails on non_existent consumer
    }

    #[test]
    fn only_consumer_can_preform_actions() {
        let (app, meta_staking_addr) = proper_instantiate();

        // Fund contract fails on non_existent consumer
    }

    #[test]
    fn cannot_delegate_more_than_consumer_has_allocated() {
        let (app, meta_staking_addr) = proper_instantiate();
    }
}
