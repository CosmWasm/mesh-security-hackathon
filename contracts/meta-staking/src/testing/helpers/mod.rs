use utils::AddrHelper;

pub mod utils;
pub mod instantiate;
pub mod execute;
pub mod queries;

pub const ADMIN: AddrHelper = AddrHelper::new("admin");
pub const USER: AddrHelper = AddrHelper::new("user");
pub const VALIDATOR: AddrHelper = AddrHelper::new("validator");

pub const NATIVE_DENOM: &str = "denom";
