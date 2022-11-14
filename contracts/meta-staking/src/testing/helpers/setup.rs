use cosmwasm_std::{testing::mock_dependencies, coins, coin};

use crate::msg::SudoMsg;

use super::{NATIVE_DENOM, contract_wrapper::{ContractWrapper, Sudo}, CONSUMER_1};

pub fn setup_contract() -> ContractWrapper{
    let mut deps = mock_dependencies();

    // Set the bonded denom
    deps.querier.update_staking(NATIVE_DENOM, &[], &[]);

    // init meta-staking contract
    let mut contract = ContractWrapper::init(deps);

    // fund the contract
    contract.fund_contract(coins(100000, NATIVE_DENOM));
    contract
}

pub fn setup_contract_with_consumer() -> ContractWrapper{
    let mut contract_wrapper = setup_contract();

    // Add consumer
    contract_wrapper
        .sudo(SudoMsg::AddConsumer {
            consumer_address: CONSUMER_1.addr().to_string(),
            funds_available_for_staking: coin(10000, NATIVE_DENOM),
        })
        .unwrap();

    contract_wrapper
}
