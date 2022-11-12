use cosmwasm_std::{
    coin, coins,
    testing::{mock_dependencies_with_balances, mock_env, mock_info},
    Addr, CosmosMsg, Decimal, FullDelegation, Uint128, Validator, to_binary, WasmMsg, Empty,
};
use cw_multi_test::Wasm;
use mesh_apis::ConsumerExecuteMsg;

use crate::{
    contract::{execute, instantiate, sudo},
    msg::{ExecuteMsg, InstantiateMsg}, error::ContractError,
};

use super::helpers::{ADMIN, NATIVE_DENOM, VALIDATOR};

#[test]
fn withdraw_to_customer() {
    let consumer_addr = Addr::unchecked("consumer");
    let consumer_info = mock_info(consumer_addr.as_str(), &[]);
    let admin_info = mock_info(&ADMIN.to_string(), &[]);
    let mut deps = mock_dependencies_with_balances(&[
        (consumer_addr.as_str(), &[coin(100000, NATIVE_DENOM)]),
        (&ADMIN.to_string(), &[coin(100000, NATIVE_DENOM)]),
    ]);

    // Set the bonded denom
    deps.querier.update_staking(
        NATIVE_DENOM,
        &[Validator {
            address: VALIDATOR.to_string(),
            commission: Decimal::zero(),
            max_commission: Decimal::one(),
            max_change_rate: Decimal::one(),
        }],
        &[],
    );

    // init meta-staking and fund it
    let env = mock_env();
    instantiate(
        deps.as_mut(),
        env.clone(),
        admin_info.clone(),
        InstantiateMsg {},
    )
    .unwrap();
    let meta_staking_addr = env.contract.address;
    deps.querier
        .update_balance(meta_staking_addr.clone(), coins(100000, NATIVE_DENOM));

    // add consumer
    sudo(
        deps.as_mut(),
        mock_env(),
        crate::msg::SudoMsg::AddConsumer {
            consumer_address: consumer_addr.to_string(),
            funds_available_for_staking: coin(10000, NATIVE_DENOM),
        },
    )
    .unwrap();

    // Delegate funds
    execute(
        deps.as_mut(),
        mock_env(),
        consumer_info,
        ExecuteMsg::Delegate {
            validator: VALIDATOR.to_string(),
            amount: Uint128::from(1000_u128),
        },
    )
    .unwrap();

    // set delegation with rewards
    deps.querier.update_staking(
        NATIVE_DENOM,
        &[],
        &[FullDelegation {
            delegator: meta_staking_addr.clone(),
            validator: VALIDATOR.to_string(),
            amount: coin(10000, NATIVE_DENOM),
            can_redelegate: coin(10000, NATIVE_DENOM),
            accumulated_rewards: coins(1000, NATIVE_DENOM),
        }],
    );

    // withdraw rewards from validator
    execute(
        deps.as_mut(),
        mock_env(),
        admin_info.clone(),
        ExecuteMsg::WithdrawDelegatorReward {
            validator: VALIDATOR.to_string(),
        },
    )
    .unwrap();

    // Withdraw to consumer
    let res = execute(
        deps.as_mut(),
        mock_env(),
        admin_info.clone(),
        crate::msg::ExecuteMsg::WithdrawToCostumer {
            consumer: consumer_addr.to_string(),
            validator: VALIDATOR.to_string(),
        },
    )
    .unwrap();

    // Make sure we send the correct msg with the correct funds
    assert_eq!(
        res.messages[0].msg,
        CosmosMsg::Wasm::<Empty>(WasmMsg::Execute {
            contract_addr: consumer_addr.to_string(),
            msg: to_binary(&ConsumerExecuteMsg::MeshConsumerRecieveRewardsMsg { validator: VALIDATOR.to_string() }).unwrap(),
            funds: coins(1000, NATIVE_DENOM)
        })
    );

    // try to withdraw again, should fail
    let err = execute(
        deps.as_mut(),
        mock_env(),
        admin_info,
        crate::msg::ExecuteMsg::WithdrawToCostumer {
            consumer: "random".to_string(),
            validator: VALIDATOR.to_string(),
        },
    )
    .unwrap_err();

    assert!(matches!(err, ContractError::NoConsumer {}));
}
