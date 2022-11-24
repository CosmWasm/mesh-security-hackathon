use cosmwasm_std::{
    coin, coins,
    testing::{mock_env, mock_info},
    to_binary, CosmosMsg, Decimal, FullDelegation, Uint128, Validator, WasmMsg,
};

use crate::{
    contract::execute,
    msg::ExecuteMsg,
    testing::utils::{
        executes::{undelegate, withdraw_rewards},
        queries::{query_consumer, query_rewards},
        setup::{setup_with_contracts, setup_with_multiple_delegations},
    },
    ContractError,
};

use mesh_testing::{
    constants::{CREATOR_ADDR, NATIVE_DENOM, VALIDATOR},
    macros::assert_error,
};

use super::utils::setup::{setup_unit_with_contract, setup_unit_with_delegation};

#[test]
fn verify_rewards() {
    let (mut app, meta_staking_addr, mesh_consumer_addr_1, mesh_consumer_addr_2) =
        setup_with_multiple_delegations();

    // move block year a head
    app.update_block(|block| block.time = block.time.plus_seconds(60 * 60 * 24 * 365));

    let total_rewards = query_rewards(&app, meta_staking_addr.as_str(), VALIDATOR).unwrap();

    withdraw_rewards(
        &mut app,
        meta_staking_addr.as_str(),
        CREATOR_ADDR,
        VALIDATOR,
    )
    .unwrap();

    // Make sure we withdrew the rewards and there are none left.
    let no_rewards = query_rewards(&app, meta_staking_addr.as_str(), VALIDATOR);
    assert!(no_rewards.is_none());

    // We undelegate to force reward calculation for both consumers
    undelegate(
        &mut app,
        meta_staking_addr.as_str(),
        mesh_consumer_addr_1.as_str(),
        VALIDATOR,
        Uint128::one(),
    )
    .unwrap();

    undelegate(
        &mut app,
        meta_staking_addr.as_str(),
        mesh_consumer_addr_2.as_str(),
        VALIDATOR,
        Uint128::one(),
    )
    .unwrap();

    // We query consumers to see how much rewards they have pending
    let consumer_1 = query_consumer(
        &app,
        meta_staking_addr.as_str(),
        mesh_consumer_addr_1.as_str(),
    )
    .unwrap();
    let consumer_2 = query_consumer(
        &app,
        meta_staking_addr.as_str(),
        mesh_consumer_addr_2.as_str(),
    )
    .unwrap();

    let rewards_1 = consumer_1.pending_to_u128().unwrap();
    let rewards_2 = consumer_2.pending_to_u128().unwrap();

    // make sure rewards not equal, so we confirm its not false positive test.
    // If they are equal our calculation can be off.
    assert_ne!(rewards_1, rewards_2);
    // -1 is for leftover from rounding (left as pending)
    // When we delegate we delegate not round numbers to get some leftovers.
    assert_eq!(rewards_1 + rewards_2, total_rewards.u128() - 1);
}

#[test]
fn try_withdraw_no_delegations() {
    let (mut app, meta_staking_addr, _) = setup_with_contracts();

    let err = withdraw_rewards(
        &mut app,
        meta_staking_addr.as_str(),
        CREATOR_ADDR,
        VALIDATOR,
    );

    assert_error!(err, ContractError::NoDelegationsForValidator {});
}

// TODO: can't use multi-test to test withdraw_to_consumer because it doesn't support IBC calls
// We do unit testing for now.
#[test]
fn withdraw_to_consumer() {
    let (mut deps, meta_staking_addr, consumer_addr) = setup_unit_with_delegation();
    let admin_info = mock_info(CREATOR_ADDR, &[]);

    // Set rewards so we have something to send away
    deps.querier.update_staking(
        NATIVE_DENOM,
        &[Validator {
            address: VALIDATOR.to_string(),
            commission: Decimal::zero(),
            max_commission: Decimal::one(),
            max_change_rate: Decimal::one(),
        }],
        &[FullDelegation {
            delegator: meta_staking_addr.clone(),
            validator: VALIDATOR.to_string(),
            amount: coin(10000, NATIVE_DENOM),
            can_redelegate: coin(10000, NATIVE_DENOM),
            accumulated_rewards: coins(1000, NATIVE_DENOM),
        }],
    );

    // Withdraw rewards from validator
    execute(
        deps.as_mut(),
        mock_env(),
        admin_info.clone(),
        ExecuteMsg::WithdrawDelegatorReward {
            validator: VALIDATOR.to_string(),
        },
    )
    .unwrap();

    // Withdraw/send to consumer
    let res = execute(
        deps.as_mut(),
        mock_env(),
        admin_info.clone(),
        ExecuteMsg::WithdrawToCostumer {
            consumer: consumer_addr.to_string(),
            validator: VALIDATOR.to_string(),
        },
    )
    .unwrap();

    // Make sure the msg we send away is the one we expect
    assert_eq!(
        res.messages[0].msg,
        CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: consumer_addr.to_string(),
            msg: to_binary(
                &mesh_apis::ConsumerExecuteMsg::MeshConsumerRecieveRewardsMsg {
                    validator: VALIDATOR.to_string()
                }
            )
            .unwrap(),
            funds: coins(1000, NATIVE_DENOM)
        })
    );
}

#[test]
fn try_withdraw_no_rewards() {
    let (mut deps, meta_staking_addr, consumer_addr) = setup_unit_with_delegation();
    let admin_info = mock_info(CREATOR_ADDR, &[]);

    // Setup delegation with 0 rewards
    deps.querier.update_staking(
        NATIVE_DENOM,
        &[Validator {
            address: VALIDATOR.to_string(),
            commission: Decimal::zero(),
            max_commission: Decimal::one(),
            max_change_rate: Decimal::one(),
        }],
        &[FullDelegation {
            delegator: meta_staking_addr.clone(),
            validator: VALIDATOR.to_string(),
            amount: coin(10000, NATIVE_DENOM),
            can_redelegate: coin(10000, NATIVE_DENOM),
            accumulated_rewards: coins(0, NATIVE_DENOM),
        }],
    );

    let err = execute(
        deps.as_mut(),
        mock_env(),
        admin_info.clone(),
        ExecuteMsg::WithdrawDelegatorReward {
            validator: VALIDATOR.to_string(),
        },
    )
    .unwrap_err();

    assert_eq!(err, ContractError::ZeroRewardsToSend {});

    let err = execute(
        deps.as_mut(),
        mock_env(),
        admin_info.clone(),
        ExecuteMsg::WithdrawToCostumer {
            consumer: consumer_addr.to_string(),
            validator: VALIDATOR.to_string(),
        },
    )
    .unwrap_err();

    assert_eq!(err, ContractError::ZeroRewardsToSend {});
}

#[test]
fn try_withdraw_no_consumer() {
    let (mut deps, _) = setup_unit_with_contract();
    let admin_info = mock_info(CREATOR_ADDR, &[]);

    let err = execute(
        deps.as_mut(),
        mock_env(),
        admin_info.clone(),
        ExecuteMsg::WithdrawDelegatorReward {
            validator: VALIDATOR.to_string(),
        },
    )
    .unwrap_err();

    assert_eq!(err, ContractError::NoDelegationsForValidator {});

    let err = execute(
        deps.as_mut(),
        mock_env(),
        admin_info.clone(),
        ExecuteMsg::WithdrawToCostumer {
            consumer: "consumer_addr".to_string(),
            validator: VALIDATOR.to_string(),
        },
    )
    .unwrap_err();

    assert_eq!(err, ContractError::NoConsumer {});
}
