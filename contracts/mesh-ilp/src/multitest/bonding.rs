use super::suite::SuiteBuilder;
use crate::multitest::suite::Suite;

#[test]
fn bond_and_unbond_same_tokens() {
    let actor = "jakub";
    let start = 1234000u128;

    let mut suite: Suite = SuiteBuilder::new().with_funds(actor, start).build();

    assert_eq!(suite.balance(actor).unwrap().u128(), start);

    // bond too much
    suite.bond(actor, 999999999999).unwrap_err();
    // bond a bit
    let bond = 234000u128;
    suite.bond(actor, bond).unwrap();

    // query amounts
    assert_eq!(suite.balance(actor).unwrap().u128(), start - bond);
    let bal = suite.ilp_balance(actor).unwrap();
    assert_eq!(bal.bonded.u128(), bond);
    assert_eq!(bal.free.u128(), bond);
    assert_eq!(bal.claims.len(), 0);

    // unbond some back
    let unbond = 123000u128;
    suite.unbond(actor, unbond).unwrap();

    // query amounts
    assert_eq!(suite.balance(actor).unwrap().u128(), start - bond + unbond);
    let bal = suite.ilp_balance(actor).unwrap();
    assert_eq!(bal.bonded.u128(), bond - unbond);
    assert_eq!(bal.free.u128(), bond - unbond);
    assert_eq!(bal.claims.len(), 0);
}
