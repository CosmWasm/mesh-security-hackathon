use self::helpers::AddrHelper;

//mod integration;
mod helpers;
mod instantiate;
mod execute;
mod queries;
mod tests;

const ADMIN: AddrHelper = AddrHelper::new("admin");
const USER: AddrHelper = AddrHelper::new("user");
const CONSUMER: AddrHelper = AddrHelper::new("consumer");
const CONSUMER2: AddrHelper = AddrHelper::new("consumer2");
const VALIDATOR: AddrHelper = AddrHelper::new("validator");

const NATIVE_DENOM: &str = "denom";
