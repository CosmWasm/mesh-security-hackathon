use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Uint128};

use crate::ContractError;
use cw_storage_plus::{Item, Map};

#[cw_serde]
pub struct Config {
    pub denom: String,
}

#[cw_serde]
pub struct Balance {
    pub bonded: Uint128,
    pub claims: Vec<LeinAddr>,
}

impl Balance {
    #[allow(dead_code)]
    pub fn new(bonded: u128) -> Self {
        Balance {
            bonded: bonded.into(),
            claims: vec![],
        }
    }
}

impl Default for Balance {
    fn default() -> Self {
        Balance {
            bonded: Uint128::zero(),
            claims: vec![],
        }
    }
}

#[cw_serde]
pub struct LeinAddr {
    pub leinholder: Addr,
    pub amount: Uint128,
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
        let after = self.claims[pos]
            .amount
            .checked_sub(amount)
            .map_err(|_| ContractError::InsufficientLein)?;
        if !after.is_zero() {
            self.claims[pos].amount = after;
        } else {
            self.claims.remove(pos);
        }
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

#[cfg(test)]
mod tests {
    use super::*;

    use test_case::test_case;

    #[test_case(123456789012345u128; "Empty claims works")]
    #[test_case(0u128; "Zero balance valid")]
    fn no_claims(bonded: u128) {
        let balance = Balance::new(bonded);
        assert_eq!(balance.free().u128(), bonded);
    }

    #[test_case(123_000, &[5000, 6000, 23000], 89_000; "free deducts claims from one addr")]
    #[test_case(87_000, &[74_000, 13_000], 0; "can deduct all")]
    fn claims_add(bonded: u128, add_claims: &[u128], free: u128) {
        let leinholder = Addr::unchecked("foo");
        let mut balance = Balance::new(bonded);
        for claim in add_claims {
            balance
                .add_claim(&leinholder, Uint128::new(*claim))
                .unwrap();
        }
        assert_eq!(balance.free().u128(), free);
        assert_eq!(balance.claims.len(), 1);
    }

    #[test_case(123_000, &[&[12000], &[5000, 8000]], 110_000; "free takes max from one leinholder as claimed")]
    #[test_case(250_000, &[&[12000, 17000], &[5000, 8000, 1000], &[8000, 22000, 70000]], 150_000; "handles many holders")]
    fn max_from_multiple_clains(bonded: u128, add_claims: &[&[u128]], free: u128) {
        let mut balance = Balance::new(bonded);
        for (i, claims) in add_claims.iter().enumerate() {
            let leinholder = Addr::unchecked(format! {"Owner {}", i});
            for claim in *claims {
                balance
                    .add_claim(&leinholder, Uint128::new(*claim))
                    .unwrap();
            }
        }
        assert_eq!(balance.free().u128(), free);
        assert_eq!(balance.claims.len(), add_claims.len());
    }

    #[test_case(200_000, (180_000, 80_000, 50_000), (150_000, 100_000); "100/200 bonded, 50 slashed, 50/150 bonded ")]
    #[test_case(300_000, (250_000, 70_000, 80_000), (220_000, 120_000); "180/300 bonded, 80 slashed, 100/220 bonded ")]
    fn add_release_slash(
        init_bond: u128,
        (add, release, slash): (u128, u128, u128),
        (bonded, free): (u128, u128),
    ) {
        let leinholder = Addr::unchecked("foo");
        let mut balance = Balance::new(init_bond);
        balance.add_claim(&leinholder, add.into()).unwrap();
        balance.release_claim(&leinholder, release.into()).unwrap();
        balance.slash_claim(&leinholder, slash.into()).unwrap();
        assert_eq!(balance.bonded.u128(), bonded);
        assert_eq!(balance.free().u128(), free);
        assert_eq!(balance.claims.len(), 1);
    }

    #[test_case(1000, 1500)]
    #[test_case(0, 1)]
    fn cannot_claim_more_than_bonded(init: u128, add: u128) {
        let leinholder = Addr::unchecked("foo");
        let mut balance = Balance::new(init);
        let err = balance.add_claim(&leinholder, add.into());
        assert!(err.is_err())
    }

    #[test_case(2000, 1000, 1500)]
    #[test_case(2000, 0, 1)]
    fn cannot_release_more_than_added(init: u128, add: u128, release: u128) {
        let leinholder = Addr::unchecked("foo");
        let mut balance = Balance::new(init);
        if add > 0 {
            balance.add_claim(&leinholder, add.into()).unwrap();
        }
        let err = balance.release_claim(&leinholder, release.into());
        assert!(err.is_err())
    }

    #[test]
    fn delete_when_all_release() {
        let leinholder = Addr::unchecked("foo");
        let init = 12345;
        let mut balance = Balance::new(init);
        balance.add_claim(&leinholder, init.into()).unwrap();
        assert_eq!(balance.free().u128(), 0);
        assert_eq!(balance.claims.len(), 1);
        balance.release_claim(&leinholder, init.into()).unwrap();
        assert_eq!(balance.free().u128(), init);
        assert_eq!(balance.claims.len(), 0);
    }
}
