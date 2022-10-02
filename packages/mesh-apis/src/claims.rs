use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Binary, Decimal, Uint128};

#[cw_serde]
pub enum ReceiveClaimMsg {
    /// This gives the receiver access to slash part up to this much claim
    /// TODO: shall we limit Binary to a small subset, as we may need to add more logic here
    ReceiveClaim {
        owner: String,
        amount: Uint128,
        msg: Binary,
    },
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
