use cosmwasm_std::Addr;

/// Struct to allow us have Addr as const
pub struct AddrWrapper<'a>(pub &'a str);

impl<'a> AddrWrapper<'a> {
    /// Allow us to set const addrs
    pub const fn new(addr: &'a str) -> Self {
        AddrWrapper(addr)
    }

    /// Get Addr
    pub fn addr(&self) -> Addr {
        Addr::unchecked(self.0.clone())
    }

    pub fn as_str(&self) -> &str {
        self.0
    }

    pub fn to_string(&self) -> String {
        self.0.clone().to_string()
    }
}
