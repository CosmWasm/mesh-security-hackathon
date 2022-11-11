
use crate::msg::{SudoMsg as MetaStakingSudoMsg, ExecuteMsg, InstantiateMsg};
use cosmwasm_std::testing::mock_env;
use cosmwasm_std::{to_binary, Addr, Coin, Decimal, Empty, Uint128, Validator};
use cw_multi_test::{
    next_block, App, AppBuilder, BankSudo, Contract, ContractWrapper, Executor, SudoMsg, WasmSudo,
};

pub fn meta_staking_contract() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(
        crate::contract::execute,
        crate::contract::instantiate,
        crate::contract::query,
    )
    .with_sudo(crate::contract::sudo);
    Box::new(contract)
}

const USER: &str = "USER";
const ADMIN: &str = "ADMIN";
const NATIVE_DENOM: &str = "TOKEN";
const VALIDATOR: &str = "validator";

fn mock_app() -> App {
    let env = mock_env();
    AppBuilder::new().build(|router, api, storage| {
        router
            .staking
            .add_validator(
                api,
                storage,
                &env.block,
                Validator {
                    address: VALIDATOR.to_string(),
                    commission: Decimal::zero(),
                    max_commission: Decimal::one(),
                    max_change_rate: Decimal::one(),
                },
            )
            .unwrap();

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

fn proper_instantiate() -> (App, MetaStakingContract) {
    let mut app = mock_app();
    let cw_template_id = app.store_code(meta_staking_contract());

    let msg = InstantiateMsg {};
    let cw_template_contract_addr = app
        .instantiate_contract(
            cw_template_id,
            Addr::unchecked(ADMIN),
            &msg,
            &[],
            "test",
            None,
        )
        .unwrap();

    let meta_staking_addr = MetaStakingContract(cw_template_contract_addr);

    (app, meta_staking_addr)
}

fn add_and_fund_consumer() -> (App, MetaStakingContract, Addr) {
    let (mut app, meta_staking_addr) = proper_instantiate();

    let consumer = Addr::unchecked("consumer_contract");

    // Gov funds meta-staking contract
    // This is a workaround until we have superfluid staking
    app.sudo(SudoMsg::Bank(BankSudo::Mint {
        to_address: meta_staking_addr.addr().to_string(),
        amount: vec![Coin {
            amount: Uint128::new(100000000),
            denom: NATIVE_DENOM.to_string(),
        }],
    }))
    .unwrap();

    app.update_block(next_block);

    // Gov adds consumer
    app.sudo(SudoMsg::Wasm(WasmSudo {
        contract_addr: meta_staking_addr.addr(),
        msg: to_binary(&MetaStakingSudoMsg::AddConsumer {
            consumer_address: consumer.to_string(),
            funds_available_for_staking: Coin {
                denom: NATIVE_DENOM.to_string(),
                amount: Uint128::new(100000000),
            },
        })
        .unwrap(),
    }))
    .unwrap();

    (app, meta_staking_addr, consumer)
}

#[test]
fn happy_path() {
    let (mut app, meta_staking_addr, consumer) = add_and_fund_consumer();

    let validator_addr = Addr::unchecked(VALIDATOR);

    let balance = app
        .wrap()
        .query_all_balances(meta_staking_addr.addr())
        .unwrap();
    println!("{:?}, {:?}", meta_staking_addr.addr(), balance);

    // Consumer delegates funds
    app.execute_contract(
        consumer.clone(),
        meta_staking_addr.addr(),
        &ExecuteMsg::Delegate {
            validator: validator_addr.to_string(),
            amount: Uint128::new(100000),
        },
        &[],
    )
    .unwrap();

    // Update block
    app.update_block(next_block);

    // Consumer claims rewards
    // Errors because there are no rewards to claim
    app.execute_contract(
        consumer.clone(),
        meta_staking_addr.addr(),
        &ExecuteMsg::WithdrawDelegatorReward {
            validator: validator_addr.to_string(),
        },
        &[],
    )
    .unwrap_err();

    // Consumer unbonds funds
    app.execute_contract(
        consumer,
        meta_staking_addr.addr(),
        &ExecuteMsg::Undelegate {
            validator: validator_addr.to_string(),
            amount: Uint128::new(100000),
        },
        &[],
    )
    .unwrap();
}

#[test]
fn only_consumer_can_preform_actions() {
    let (mut app, meta_staking_addr, _) = add_and_fund_consumer();

    let validator_addr = Addr::unchecked("validator");
    let random = Addr::unchecked("random");

    // Random address fails to delegates funds
    app.execute_contract(
        random,
        meta_staking_addr.addr(),
        &ExecuteMsg::Delegate {
            validator: validator_addr.to_string(),
            amount: Uint128::new(100000),
        },
        &[],
    )
    .unwrap_err();
}
