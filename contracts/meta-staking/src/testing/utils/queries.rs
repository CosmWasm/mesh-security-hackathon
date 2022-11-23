use cosmwasm_std::{StdResult, Uint128};
use cw_multi_test::App;

use crate::{
    msg::{Delegation, QueryMsg},
    state::ConsumerInfo,
};

pub fn query_delegation(
    app: &App,
    contract_addr: &str,
    consumer: &str,
    validator: &str,
) -> StdResult<Uint128> {
    let delegation = app.wrap().query_wasm_smart(
        contract_addr,
        &QueryMsg::Delegation {
            consumer: consumer.to_string(),
            validator: validator.to_string(),
        },
    )?;
    Ok(delegation)
}

pub fn query_all_delegations(
    app: &App,
    contract_addr: &str,
    consumer: &str,
) -> StdResult<Vec<Delegation>> {
    let delegations = app.wrap().query_wasm_smart(
        contract_addr,
        &QueryMsg::AllDelegations {
            consumer: consumer.to_string(),
        },
    )?;
    Ok(delegations)
}

pub fn query_consumer(app: &App, contract_addr: &str, consumer: &str) -> StdResult<ConsumerInfo> {
    let consumer = app.wrap().query_wasm_smart(
        contract_addr,
        &QueryMsg::Consumer {
            address: consumer.to_string(),
        },
    )?;
    Ok(consumer)
}

pub fn query_consumers(
    app: &App,
    contract_addr: &str,
    start: Option<String>,
    limit: Option<u32>,
) -> StdResult<ConsumerInfo> {
    let consumers = app
        .wrap()
        .query_wasm_smart(contract_addr, &&QueryMsg::Consumers { start, limit })?;
    Ok(consumers)
}

pub fn query_all_validators(
    app: &App,
    contract_addr: &str,
    consumer: &str,
    start: Option<String>,
    limit: Option<u32>,
) -> StdResult<Vec<String>> {
    let validators = app.wrap().query_wasm_smart(
        contract_addr,
        &&QueryMsg::AllValidators {
            consumer: consumer.to_string(),
            start,
            limit,
        },
    )?;
    Ok(validators)
}
