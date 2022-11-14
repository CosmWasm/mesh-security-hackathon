use cosmwasm_std::{coin, coins, testing::mock_dependencies, Uint128};

use crate::msg::{ExecuteMsg, SudoMsg};

use super::{
    contract_wrapper::{ContractWrapper, Execute, Sudo},
    CONSUMER_1, NATIVE_DENOM, VALIDATOR,
};

pub fn setup_contract() -> ContractWrapper {
    let mut deps = mock_dependencies();

    // Set the bonded denom
    deps.querier.update_staking(NATIVE_DENOM, &[], &[]);

    // init meta-staking contract
    let mut contract = ContractWrapper::init(deps);

    // fund the contract
    contract.fund_contract(coins(100000, NATIVE_DENOM));
    contract
}

pub fn setup_contract_with_consumer() -> ContractWrapper {
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

pub fn setup_contract_with_delegation() -> ContractWrapper {
    let mut contract_wrapper = setup_contract_with_consumer();

    // Add delegation
    contract_wrapper
        .execute(
            CONSUMER_1.addr().as_str(),
            ExecuteMsg::Delegate {
                validator: VALIDATOR.addr().to_string(),
                amount: Uint128::new(10000_u128),
            },
        )
        .unwrap();

    contract_wrapper
}
