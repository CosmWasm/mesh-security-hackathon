use cosmwasm_std::StdResult;
use cw_multi_test::App;
use mesh_testing::msgs::{SlasherConfigResponse, SlasherQueryMsg};

use crate::msg::{ConfigResponse, ListValidatorsResponse, QueryMsg};

pub fn query_provider_config(app: &App, contract_addr: &str) -> StdResult<ConfigResponse> {
    app.wrap()
        .query_wasm_smart(contract_addr, &QueryMsg::Config {})
}

pub fn query_slasher_config(app: &App, contract_addr: &str) -> StdResult<SlasherConfigResponse> {
    app.wrap()
        .query_wasm_smart(contract_addr, &SlasherQueryMsg::Config {})
}

pub fn query_validators(
    app: &App,
    contract_addr: &str,
    start: Option<String>,
    limit: Option<u32>,
) -> StdResult<ListValidatorsResponse> {
    app.wrap().query_wasm_smart(
        contract_addr,
        &QueryMsg::ListValidators {
            start_after: start,
            limit,
        },
    )
}
