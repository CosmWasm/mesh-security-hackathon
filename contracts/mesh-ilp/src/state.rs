use cosmwasm_std::{Addr, Uint128};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::msg::Lein;
use crate::ContractError;
use cw_storage_plus::{Item, Map};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    pub denom: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Balance {
    pub bonded: Uint128,
    pub claims: Vec<LeinAddr>,
}

impl Default for Balance {
    fn default() -> Self {
        Balance {
            bonded: Uint128::zero(),
            claims: vec![],
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct LeinAddr {
    pub leinholder: Addr,
    pub amount: Uint128,
}

impl Into<Lein> for LeinAddr {
    fn into(self) -> Lein {
        Lein {
            leinholder: self.leinholder.into_string(),
            amount: self.amount,
        }
    }
}

impl Balance {
    pub fn free(&self) -> Uint128 {
        let claimed = self
            .claims
            .iter()
            .map(|l| l.amount)
            .max()
            .unwrap_or_default();
        // note: after a slash, claimed may be larger than bonder...
        self.bonded.saturating_sub(claimed)
    }

    pub fn add_claim(&mut self, leinholder: &Addr, amount: Uint128) -> Result<(), ContractError> {
        if amount > self.bonded {
            return Err(ContractError::InsufficentBalance);
        }
        let pos = self.claims.iter().position(|c| &c.leinholder == leinholder);
        match pos {
            Some(idx) => {
                let mut current = self.claims[idx].clone();
                current.amount += amount;
                if current.amount > self.bonded {
                    return Err(ContractError::InsufficentBalance);
                }
                self.claims[idx] = current;
            }
            None => self.claims.push(LeinAddr {
                leinholder: leinholder.clone(),
                amount,
            }),
        };
        Ok(())
    }

    pub fn release_claim(
        &mut self,
        leinholder: &Addr,
        amount: Uint128,
    ) -> Result<(), ContractError> {
        let pos = self.claims.iter().position(|c| &c.leinholder == leinholder);
        let pos = pos.ok_or(ContractError::UnknownLeinholder)?;
        self.claims[pos].amount = self.claims[pos]
            .amount
            .checked_sub(amount)
            .map_err(|_| ContractError::InsufficientLein)?;
        Ok(())
    }

    pub fn slash_claim(&mut self, leinholder: &Addr, amount: Uint128) -> Result<(), ContractError> {
        let pos = self.claims.iter().position(|c| &c.leinholder == leinholder);
        let pos = pos.ok_or(ContractError::UnknownLeinholder)?;
        self.claims[pos].amount = self.claims[pos]
            .amount
            .checked_sub(amount)
            .map_err(|_| ContractError::InsufficientLein)?;
        self.bonded = self.bonded.saturating_sub(amount);
        Ok(())
    }
}

pub const CONFIG: Item<Config> = Item::new("config");
pub const BALANCES: Map<&Addr, Balance> = Map::new("balances");
