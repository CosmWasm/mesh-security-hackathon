#[cfg(not(target_arch = "wasm32"))]
pub mod contracts;

#[cfg(not(target_arch = "wasm32"))]
pub mod instantiates;
