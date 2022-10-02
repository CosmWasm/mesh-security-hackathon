use cosmwasm_schema::cw_serde;
use cosmwasm_std::Uint128;

#[cw_serde]
pub enum ClaimReceiverMsg {
    /// This gives the receiver access to slash part up to this much claim
    /// Validator specifies where the owner is directing his claim
    ReceiveClaim {
        owner: String,
        amount: Uint128,
        validator: String,
    },
}

#[cw_serde]
pub enum ClaimProviderMsg {
    /// This releases a previously received claim without slashing it
    ReleaseClaim {
        owner: String,
        amount: Uint128,
    },
    SlashClaim {
        owner: String,
        amount: Uint128,
    },
}
