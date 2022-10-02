use cosmwasm_std::Decimal;

pub enum SlashMsg {
    Slash { validator: String, amount: Decimal },
}
