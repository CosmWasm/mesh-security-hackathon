#[cfg(not(target_arch = "wasm32"))]
pub mod contracts;

#[cfg(not(target_arch = "wasm32"))]
pub mod instantiates;

pub const CREATOR_ADDR: &str = "creater_addr";
