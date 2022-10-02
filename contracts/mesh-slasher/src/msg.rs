use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Decimal;

#[cw_serde]
pub struct InstantiateMsg {
    // owner is allowed to submit evidence
    pub owner: String,
}

/// This is a mock contract
#[cw_serde]
pub enum ExecuteMsg {
    /// Owner can slash validator by X%
    SubmitEvidence { validator: String, amount: Decimal },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(ConfigResponse)]
    Config {},
}

#[cw_serde]
pub struct ConfigResponse {
    /// The address that can trigger a slash
    pub owner: String,
    /// The contract that will be slashed
    pub slashee: String,
}
