use crate::state::LeinAddr;
use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Uint128;

#[cw_serde]
pub struct InstantiateMsg {
    pub denom: String,
}

#[cw_serde]
pub enum ExecuteMsg {
    /// Places tokens in Lockup so they can be staked in multiple contracts.
    /// Must be sent in funds and proper denom
    Bond {},
    /// Withdraws tokens from Lockup.
    /// Only works if the account has sufficient funds that is not backing open claims
    Unbond { amount: Uint128 },
    /// This gives a claim on my balance to leinholder, granting it to a given validator
    /// In the case of granting a claim, the leinholder is the mesh-provider contract
    GrantClaim {
        leinholder: String,
        amount: Uint128,
        validator: String,
    },
    /// This releases a previously received claim without slashing it
    ReleaseClaim { owner: String, amount: Uint128 },
    /// This slashes a previously provided claim
    SlashClaim { owner: String, amount: Uint128 },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(BalanceResponse)]
    Balance { account: String },
}

#[cw_serde]
pub struct BalanceResponse {
    pub bonded: Uint128,
    pub free: Uint128,
    pub claims: Vec<Lein>,
}

#[cw_serde]
pub struct Lein {
    pub leinholder: String,
    pub amount: Uint128,
}

impl From<LeinAddr> for Lein {
    fn from(lein: LeinAddr) -> Self {
        Lein {
            leinholder: lein.leinholder.into_string(),
            amount: lein.amount,
        }
    }
}
