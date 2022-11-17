use mesh_testing::utils::addr_wrapper::AddrWrapper;

// unit testing setups
pub mod setup;
// integration testing setups
pub mod setup_app;

#[macro_use]
pub mod utils;
pub mod rewards;

pub const ADMIN: AddrWrapper = AddrWrapper::new("admin");
pub const CONSUMER_1: AddrWrapper = AddrWrapper::new("consumer_1");
pub const CONSUMER_2: AddrWrapper = AddrWrapper::new("consumer_2");
pub const USER: AddrWrapper = AddrWrapper::new("user");
pub const VALIDATOR: AddrWrapper = AddrWrapper::new("validator");

pub const NATIVE_DENOM: &str = "denom";

pub mod execute;
pub mod instantiate;
pub mod queries;
