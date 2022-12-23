use std::str::FromStr;

use cosmwasm_std::{
    coins,
    testing::{mock_env, mock_info},
    to_binary, Decimal, IbcMsg, Uint128, WasmMsg,
};
use mesh_apis::ClaimProviderMsg;
use mesh_ibc::ProviderMsg;
use mesh_testing::constants::{
    CHANNEL_ID, DELEGATOR_ADDR, LOCKUP_ADDR, REWARDS_IBC_DENOM, VALIDATOR,
};

use crate::{
    contract::execute,
    ibc::build_timeout,
    msg::ExecuteMsg,
    state::{ValStatus, CONFIG},
    testing::utils::{
        execute::execute_slash, helpers::add_validator, query::query_validators,
        setup_unit::setup_unit_with_channel,
    },
    ContractError,
};

use super::utils::{
    execute::execute_claim_rewards,
    helpers::{add_rewards, add_stake},
    ibc_helpers::{add_stake_unit, remove_stake_unit, update_validator_unit},
    query::query_provider_config,
    setup::setup_with_contract,
};

#[test]
fn test_execute_slash() {
    let (mut app, mesh_provider_addr) = setup_with_contract();

    let provider_config = query_provider_config(&app, mesh_provider_addr.as_str()).unwrap();
    let mesh_slasher_addr = provider_config.slasher.unwrap();

    // Add validator to contract (mimic ibc update call)
    add_validator(&mut app, mesh_provider_addr.clone());

    let validators = query_validators(&app, mesh_provider_addr.as_str(), None, None).unwrap();
    assert_eq!(validators.validators.len(), 1);

    // Do a slash
    execute_slash(
        &mut app,
        mesh_slasher_addr.as_str(),
        mesh_provider_addr.as_str(),
        VALIDATOR,
        "0.1",
        false,
    )
    .unwrap();

    // Multiplier was 1, we slashed by 0.1 (10%), new multiplier should be 0.9.
    let validators = query_validators(&app, mesh_provider_addr.as_str(), None, None).unwrap();
    assert_eq!(
        validators.validators[0].multiplier,
        Decimal::from_str("0.9").unwrap()
    );

    // Try force bond
    execute_slash(
        &mut app,
        mesh_slasher_addr.as_str(),
        mesh_provider_addr.as_str(),
        VALIDATOR,
        "0.1",
        true,
    )
    .unwrap();

    let validators = query_validators(&app, mesh_provider_addr.as_str(), None, None).unwrap();
    assert_eq!(validators.validators[0].status, ValStatus::Tombstoned);

    // Try with 0 percentage
    execute_slash(
        &mut app,
        mesh_slasher_addr.as_str(),
        mesh_provider_addr.as_str(),
        VALIDATOR,
        "0",
        false,
    )
    .unwrap_err();
}

#[test]
fn test_claim_rewards() {
    let (mut app, mesh_provider_addr) = setup_with_contract();

    // Add rewards (after calculation, we added 1000 ibc_coins)
    add_rewards(&mut app, mesh_provider_addr.clone());

    execute_claim_rewards(&mut app, mesh_provider_addr.as_str(), VALIDATOR).unwrap();

    let balance = app.wrap().query_all_balances(DELEGATOR_ADDR).unwrap();

    assert_eq!(balance, coins(1000, REWARDS_IBC_DENOM))
}

#[test]
fn test_claim_rewards_failing() {
    let (mut app, mesh_provider_addr) = setup_with_contract();

    add_validator(&mut app, mesh_provider_addr.clone());
    add_stake(&mut app, mesh_provider_addr.clone(), Decimal::zero());

    // Should error, no rewards pending
    execute_claim_rewards(&mut app, mesh_provider_addr.as_str(), VALIDATOR).unwrap_err();

    add_stake(
        &mut app,
        mesh_provider_addr.clone(),
        Decimal::from_atomics(1000_u128, 0).unwrap(),
    );
    //Shoudl error with balance too low
    execute_claim_rewards(&mut app, mesh_provider_addr.as_str(), VALIDATOR).unwrap_err();
}

#[test]
fn test_unbond() {
    let (mut deps, _) = setup_unit_with_channel(None);

    update_validator_unit(deps.as_mut(), vec![VALIDATOR.to_string()], vec![]);

    add_stake_unit(deps.as_mut(), DELEGATOR_ADDR, VALIDATOR, Uint128::new(1000)).unwrap();

    // To unstake the delegetor need to send the request
    let info = mock_info(DELEGATOR_ADDR, &[]);

    remove_stake_unit(deps.as_mut(), DELEGATOR_ADDR, VALIDATOR, Uint128::new(1000)).unwrap();

    // Update block
    let unbound_period = CONFIG.load(deps.as_mut().storage).unwrap().unbonding_period;
    let mut env = mock_env();
    env.block.time = env.block.time.plus_seconds(unbound_period + 1);

    let res = execute(deps.as_mut(), env, info, ExecuteMsg::Unbond {}).unwrap();

    assert_eq!(
        res.messages[0].msg,
        WasmMsg::Execute {
            contract_addr: LOCKUP_ADDR.to_string(),
            msg: to_binary(&ClaimProviderMsg::SlashClaim {
                owner: DELEGATOR_ADDR.to_string(),
                amount: Uint128::new(1000),
            })
            .unwrap(),
            funds: vec![]
        }
        .into()
    );

    let info = mock_info(DELEGATOR_ADDR, &[]);
    let err = execute(deps.as_mut(), mock_env(), info, ExecuteMsg::Unbond {}).unwrap_err();

    assert_eq!(err, ContractError::NothingToClaim);
}

#[test]
fn test_recieve_claim() {
    let (mut deps, _) = setup_unit_with_channel(None);

    update_validator_unit(deps.as_mut(), vec![VALIDATOR.to_string()], vec![]);

    let res = execute(
        deps.as_mut(),
        mock_env(),
        mock_info(LOCKUP_ADDR, &[]),
        ExecuteMsg::ReceiveClaim {
            owner: DELEGATOR_ADDR.to_string(),
            amount: Uint128::new(1000),
            validator: VALIDATOR.to_string(),
        },
    )
    .unwrap();

    assert_eq!(
        res.messages[0].msg,
        IbcMsg::SendPacket {
            channel_id: CHANNEL_ID.to_string(),
            data: to_binary(&ProviderMsg::Stake {
                validator: VALIDATOR.to_string(),
                amount: Uint128::new(1000),
                key: DELEGATOR_ADDR.to_string()
            })
            .unwrap(),
            timeout: build_timeout(deps.as_ref(), &mock_env()).unwrap(),
        }
        .into()
    );

    // Try with 0 amount
    let err = execute(
        deps.as_mut(),
        mock_env(),
        mock_info(LOCKUP_ADDR, &[]),
        ExecuteMsg::ReceiveClaim {
            owner: DELEGATOR_ADDR.to_string(),
            amount: Uint128::zero(),
            validator: VALIDATOR.to_string(),
        },
    )
    .unwrap_err();

    assert_eq!(err, ContractError::ZeroAmount);

    // Try with unknown validator
    let err = execute(
        deps.as_mut(),
        mock_env(),
        mock_info(LOCKUP_ADDR, &[]),
        ExecuteMsg::ReceiveClaim {
            owner: DELEGATOR_ADDR.to_string(),
            amount: Uint128::new(1000),
            validator: "some_validator".to_string(),
        },
    )
    .unwrap_err();

    assert_eq!(
        err,
        ContractError::UnknownValidator("some_validator".to_string())
    )
}

#[test]
fn test_unstake() {
    let (mut deps, _) = setup_unit_with_channel(None);

    update_validator_unit(deps.as_mut(), vec![VALIDATOR.to_string()], vec![]);

    add_stake_unit(deps.as_mut(), DELEGATOR_ADDR, VALIDATOR, Uint128::new(1000)).unwrap();

    let res = execute(
        deps.as_mut(),
        mock_env(),
        mock_info(DELEGATOR_ADDR, &[]),
        ExecuteMsg::Unstake {
            amount: Uint128::new(1000),
            validator: VALIDATOR.to_string(),
        },
    )
    .unwrap();

    // No slash, so only 1 msg should exist
    assert_eq!(res.messages.len(), 1);
    assert_eq!(
        res.messages[0].msg,
        IbcMsg::SendPacket {
            channel_id: CHANNEL_ID.to_string(),
            data: to_binary(&ProviderMsg::Unstake {
                validator: VALIDATOR.to_string(),
                amount: Uint128::new(1000),
                key: DELEGATOR_ADDR.to_string()
            })
            .unwrap(),
            timeout: build_timeout(deps.as_ref(), &mock_env()).unwrap(),
        }
        .into()
    );

    // Try with 0 amount
    let err = execute(
        deps.as_mut(),
        mock_env(),
        mock_info(DELEGATOR_ADDR, &[]),
        ExecuteMsg::Unstake {
            amount: Uint128::zero(),
            validator: VALIDATOR.to_string(),
        },
    )
    .unwrap_err();
    assert_eq!(err, ContractError::ZeroAmount);

    // Try with removed validator
    update_validator_unit(deps.as_mut(), vec![], vec![VALIDATOR.to_string()]);
    let err = execute(
        deps.as_mut(),
        mock_env(),
        mock_info(DELEGATOR_ADDR, &[]),
        ExecuteMsg::Unstake {
            amount: Uint128::new(1000),
            validator: VALIDATOR.to_string(),
        },
    )
    .unwrap_err();
    assert_eq!(err, ContractError::RemovedValidator(VALIDATOR.to_string()));
}
