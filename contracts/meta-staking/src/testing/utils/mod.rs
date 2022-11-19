use mesh_testing::utils::addr_wrapper::AddrWrapper;

// unit testing setups
pub mod setup;
// integration testing setups
pub mod setup_app;

pub mod rewards;

pub const CONSUMER_1: AddrWrapper = AddrWrapper::new("consumer_1");
pub const CONSUMER_2: AddrWrapper = AddrWrapper::new("consumer_2");
pub const VALIDATOR: AddrWrapper = AddrWrapper::new("validator");
