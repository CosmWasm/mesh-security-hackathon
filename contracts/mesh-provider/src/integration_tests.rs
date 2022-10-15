#[cfg(test)]
mod tests {
    use cosmwasm_std::{to_binary, Addr, Coin, Empty, Uint128};
    use cw_multi_test::{App, AppBuilder, Contract, ContractWrapper, Executor};

    use crate::msg::{ConfigResponse, ConsumerInfo, InstantiateMsg, QueryMsg, SlasherInfo};

    pub fn contract_provider() -> Box<dyn Contract<Empty>> {
        let contract = ContractWrapper::new(
            crate::contract::execute,
            crate::contract::instantiate,
            crate::contract::query,
        )
        .with_reply(crate::contract::reply);
        Box::new(contract)
    }

    pub fn contract_slasher() -> Box<dyn Contract<Empty>> {
        let contract = ContractWrapper::new(
            mesh_slasher::contract::execute,
            mesh_slasher::contract::instantiate,
            mesh_slasher::contract::query,
        );
        Box::new(contract)
    }

    const USER: &str = "user";
    const ADMIN: &str = "admin";
    const NATIVE_DENOM: &str = "denom";
    const CONNECTION: &str = "connection-1";

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

    fn proper_instantiate() -> (App, Addr) {
        let mut app = mock_app();
        let cw_provider_id = app.store_code(contract_provider());
        let cw_slasher_id = app.store_code(contract_slasher());

        let msg = InstantiateMsg {
            consumer: ConsumerInfo {
                connection_id: CONNECTION.to_string(),
            },
            slasher: SlasherInfo {
                code_id: cw_slasher_id,
                msg: to_binary(&mesh_slasher::msg::InstantiateMsg {
                    owner: USER.to_string(),
                })
                .unwrap(),
            },
            lockup: "lockup_contract".to_string(),
            unbonding_period: 86400 * 14,
            denom: "transfer/channel-0/ucosm".to_string(),
        };
        let provider_addr = app
            .instantiate_contract(
                cw_provider_id,
                Addr::unchecked(ADMIN),
                &msg,
                &[],
                "test",
                Some(ADMIN.to_string()),
            )
            .unwrap();

        (app, provider_addr)
    }

    #[test]
    fn instantiate_works() {
        let (app, provider_addr) = proper_instantiate();

        // check provider config proper
        let cfg: ConfigResponse = app
            .wrap()
            .query_wasm_smart(&provider_addr, &QueryMsg::Config {})
            .unwrap();
        assert_eq!(cfg.consumer.connection_id.as_str(), CONNECTION);
        let slasher_addr = cfg.slasher.unwrap();

        // query slasher config
        let cfg: mesh_slasher::msg::ConfigResponse = app
            .wrap()
            .query_wasm_smart(&slasher_addr, &mesh_slasher::msg::QueryMsg::Config {})
            .unwrap();
        assert_eq!(cfg.owner, USER.to_string());
        assert_eq!(cfg.slashee, provider_addr.to_string());

        // check the admin is set for provider
        let info = app.wrap().query_wasm_contract_info(&provider_addr).unwrap();
        assert_eq!(info.admin, Some(ADMIN.into()));
        // and provider is admin of slasher
        let info = app.wrap().query_wasm_contract_info(&slasher_addr).unwrap();
        assert_eq!(info.admin, Some(provider_addr.into()));
    }
}
