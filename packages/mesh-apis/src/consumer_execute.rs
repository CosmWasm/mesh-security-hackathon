use cosmwasm_schema::cw_serde;

#[cw_serde]
pub enum ConsumerExecuteMsg {
    MeshConsumerRecieveRewardsMsg {
        validator: String
    },
}
