use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Decimal;

#[cw_serde]
pub struct InstantiateMsg {}

/// This is a mock contract
#[cw_serde]
pub enum ExecuteMsg {
    /// Owner can slash validator by X%
    SubmitEvidence {
        validator: String,
        amount: Decimal,
    }
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    Owner{}
}

#[cw_serde]
pub struct OwnerResponse {
    pub owner: String,
}
