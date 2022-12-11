use cosmwasm_std::StdResult;
use cw_multi_test::App;
use mesh_testing::msgs::{SlasherConfigResponse, SlasherQueryMsg};

use crate::msg::{ConfigResponse, QueryMsg};

pub fn query_provider_config(app: &App, contract_addr: &str) -> StdResult<ConfigResponse> {
    app.wrap()
        .query_wasm_smart(contract_addr, &QueryMsg::Config {})
}

pub fn query_slasher_config(app: &App, contract_addr: &str) -> StdResult<SlasherConfigResponse> {
    app.wrap()
        .query_wasm_smart(contract_addr, &SlasherQueryMsg::Config {})
}
