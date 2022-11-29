use cosmwasm_schema::write_api;

use mesh_apis::StakingExecuteMsg as ExecuteMsg;
use meta_staking::msg::{InstantiateMsg, QueryMsg};

fn main() {
    write_api! {
        instantiate: InstantiateMsg,
        execute: ExecuteMsg,
        query: QueryMsg,
    }
}
