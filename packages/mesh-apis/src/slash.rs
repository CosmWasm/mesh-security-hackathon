use cosmwasm_schema::cw_serde;
use cosmwasm_std::Decimal;

#[cw_serde]
pub enum SlashMsg {
    Slash { validator: String, amount: Decimal },
}
