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

    // cannot unbond more than you have
    suite.unbond(actor, start).unwrap_err();
}

#[test]
fn grant_and_release_tokens() {
    let actor = "jakub";
    let validator = "val";
    let start = 1234000u128;

    let mut suite: Suite = SuiteBuilder::new()
        .with_funds(actor, start)
        .with_denom("ujuno")
        .build();

    // bond all
    suite.bond(actor, start).unwrap();
    // proivde a claim
    let grant = 777000u128;
    suite.grant_claim(actor, grant, validator).unwrap();

    // cannot unbond all
    suite.unbond(actor, start).unwrap_err();
    // can only unbond remainder
    suite.unbond(actor, start - grant).unwrap();
    assert_eq!(suite.balance(actor).unwrap().u128(), start - grant);

    // query amounts
    let bal = suite.ilp_balance(actor).unwrap();
    assert_eq!(bal.bonded.u128(), grant);
    assert_eq!(bal.free.u128(), 0);
    assert_eq!(bal.claims.len(), 1);

    // release portion of a grant
    let release = 500_000u128;
    // not too much
    suite.release_claim(actor, grant + 1).unwrap_err();
    // just right
    suite.release_claim(actor, release).unwrap();

    // query amounts
    let bal = suite.ilp_balance(actor).unwrap();
    assert_eq!(bal.bonded.u128(), grant);
    assert_eq!(bal.free.u128(), release);
    assert_eq!(bal.claims.len(), 1);

    // can only unbond remainder
    suite.unbond(actor, release + 1).unwrap_err();
    suite.unbond(actor, release).unwrap();
}

#[test]
fn slashing_tokens() {
    let actor = "jakub";
    let validator = "val";
    let start = 1234000u128;

    let mut suite: Suite = SuiteBuilder::new().with_funds(actor, start).build();

    // bond all
    suite.bond(actor, start).unwrap();
    // proivde a claim
    let grant = 777000u128;
    suite.grant_claim(actor, grant, validator).unwrap();

    // slash some
    let slash = 300_000u128;
    let release = grant - slash;
    suite.slash_claim(actor, slash).unwrap();

    // query amounts
    let bal = suite.ilp_balance(actor).unwrap();
    assert_eq!(bal.bonded.u128(), start - slash);
    assert_eq!(bal.free.u128(), start - grant);
    assert_eq!(bal.claims.len(), 1);

    // release rest and unbond
    suite.release_claim(actor, release).unwrap();
    let bal = suite.ilp_balance(actor).unwrap();
    assert_eq!(bal.bonded.u128(), start - slash);
    assert_eq!(bal.free.u128(), start - slash);
    assert_eq!(bal.claims.len(), 0);

    suite.unbond(actor, start - slash + 1).unwrap_err();
    suite.unbond(actor, start - slash).unwrap();
    assert_eq!(suite.balance(actor).unwrap().u128(), start - slash);
}
