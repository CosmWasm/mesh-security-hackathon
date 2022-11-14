use utils::AddrHelper;

pub mod utils;
pub mod instantiate;
pub mod execute;
pub mod queries;

pub mod contract_wrapper;
pub mod setup;

pub const ADMIN: AddrHelper = AddrHelper::new("admin");
pub const CONSUMER_1: AddrHelper = AddrHelper::new("consumer_1");
pub const CONSUMER_2: AddrHelper = AddrHelper::new("consumer_2");
pub const USER: AddrHelper = AddrHelper::new("user");
pub const VALIDATOR: AddrHelper = AddrHelper::new("validator");

pub const NATIVE_DENOM: &str = "denom";
