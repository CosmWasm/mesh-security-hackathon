use cosmwasm_std::Addr;
use std::str::FromStr;

use anyhow::Result as AnyResult;
use cosmwasm_std::Decimal;
use cw_multi_test::{App, AppResponse, Executor};
use mesh_testing::{addr, constants::VALIDATOR};

use crate::msg::ExecuteMsg;

pub fn execute_slash(
    app: &mut App,
    slasher_addr: &str,
    contract_addr: &str,
) -> AnyResult<AppResponse> {
    app.execute_contract(
        addr!(slasher_addr),
        addr!(contract_addr),
        &ExecuteMsg::Slash {
            validator: VALIDATOR.to_string(),
            percentage: Decimal::from_str("0.1").unwrap(),
            force_unbond: false,
        },
        &[],
    )
}