use utils::addr_wrapper::AddrWrapper;

pub mod utils;

#[cfg(not(target_arch = "wasm32"))]
#[cfg(feature = "unit")]
pub mod unit_wrapper;

#[cfg(not(target_arch = "wasm32"))]
#[cfg(feature = "multi")]
pub mod app_wrapper;

/// Native token for default uses
pub const NATIVE_DENOM: &str = "denom";
/// Default owner of contracts if you don't care about who is the admin of the contract.
pub const ADMIN: AddrWrapper = AddrWrapper::new("admin");
