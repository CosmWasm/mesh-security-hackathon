use cosmwasm_std::Addr;
use cw_multi_test::App;

use crate::state::{Validator, VALIDATORS};

use mesh_testing::{constants::VALIDATOR, multitest_helpers::update_storage};

/// Function for multi-test to add validator to storage directly and by-pass
/// the IBC call that is needed to do so.
pub fn add_validator(app: &mut App, addr: Addr) {
    update_storage(app, addr.as_bytes(), &mut |storage| {
        VALIDATORS
            .save(storage, VALIDATOR, &Validator::new())
            .unwrap();
    });
}
