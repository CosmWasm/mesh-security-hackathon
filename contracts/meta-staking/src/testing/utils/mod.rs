use mesh_testing::utils::addr_wrapper::AddrWrapper;
#[macro_use]
pub mod utils;
pub mod execute;
pub mod instantiate;
pub mod queries;

// std::testing stuff
pub mod setup;
//cw-multi-test stuff
pub mod setup_app;



pub const ADMIN: AddrWrapper = AddrWrapper::new("admin");
pub const CONSUMER_1: AddrWrapper = AddrWrapper::new("consumer_1");
pub const CONSUMER_2: AddrWrapper = AddrWrapper::new("consumer_2");
pub const USER: AddrWrapper = AddrWrapper::new("user");
pub const VALIDATOR: AddrWrapper = AddrWrapper::new("validator");

pub const NATIVE_DENOM: &str = "denom";
