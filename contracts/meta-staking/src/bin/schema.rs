use cosmwasm_schema::write_api;

use meta_staking::msg::{InstantiateMsg, QueryMsg};
use mesh_apis::StakingExecuteMsg as ExecuteMsg;

fn main() {
    write_api! {
        instantiate: InstantiateMsg,
        execute: ExecuteMsg,
        query: QueryMsg,
    }
}
