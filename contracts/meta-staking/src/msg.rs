use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Addr, Coin, Uint128};

use crate::state::ConsumerInfo;

// mesh-consumer msg to receive rewards
#[cw_serde]
pub struct MeshConsumerRecieveRewardsMsg {
    pub rewards_by_validator: Vec<(String, Vec<Coin>)>,
}

#[cw_serde]
pub struct InstantiateMsg {}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    /// AllDelegations will return all delegations by the consumer
    #[returns(Vec<Delegation>)]
    AllDelegations { consumer: String },
    /// Returns an individual consumer
    #[returns(ConsumerInfo)]
    Consumer { address: String },
    /// Returns list of consumers
    #[returns(Vec<Addr>)]
    Consumers {
        start: Option<String>,
        limit: Option<u32>,
    },
    /// Delegation will return more detailed info on a particular
    /// delegation, defined by delegator/validator pair
    #[returns(Uint128)]
    Delegation { consumer: String, validator: String },
    /// Returns all validators the consumer delegates to.
    ///
    /// The query response type is `AllValidatorsResponse`.
    #[returns(Vec<String>)]
    AllValidators {
        consumer: String,
        start: Option<String>,
        limit: Option<u32>,
    },
}

#[cw_serde]
pub struct Delegation {
    pub validator: String,
    pub delegation: Uint128,
}
