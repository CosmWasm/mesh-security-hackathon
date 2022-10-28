use cosmwasm_schema::write_api;
use mesh_apis::ConsumerExecuteMsg;

use mesh_consumer::msg::{InstantiateMsg, QueryMsg};

fn main() {
    write_api! {
        instantiate: InstantiateMsg,
        execute: ConsumerExecuteMsg,
        query: QueryMsg,
    }
}
