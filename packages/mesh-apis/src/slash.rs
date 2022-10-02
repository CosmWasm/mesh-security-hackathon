use cosmwasm_schema::cw_serde;
use cosmwasm_std::Decimal;

#[cw_serde]
pub enum SlashMsg {
    Slash {
        /// which validator to slash
        validator: String,
        /// what percentage we should slash all stakers
        percentage: Decimal,
        /// do we forcibly unbond this validator on the provider side,
        /// regardless of the behavior of the consumer?
        force_unbond: bool,
    },
}
