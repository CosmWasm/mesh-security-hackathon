use cosmwasm_schema::cw_serde;
use cosmwasm_std::Uint128;

#[cw_serde]
pub enum ConsumerExecuteMsg {
    MeshConsumerRecieveRewardsMsg {
        rewards_by_validator: Vec<(String, Uint128)>,
    },
}
