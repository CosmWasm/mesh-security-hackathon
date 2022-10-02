use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Decimal, Uint128};

#[cw_serde]
pub enum ReceiveClaimMsg {
    /// This gives the receiver access to slash part up to this much claim
    ReceiveClaim { owner: String, amount: Uint128 },
}

#[cw_serde]
pub enum ProvideClaimMsg {
    /// This releases a previously received claim without slashing it
    ReleaseClaim {
        owner: String,
        amount: Uint128,
    },
    SlashClaim {
        owner: String,
        percentage: Decimal,
    },
}
