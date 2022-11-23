/// Assert the error is the error we
///
/// Result<AppResponse, Error> === ContractError::Example{}
#[macro_export]
macro_rules! assert_error {
    ($x:expr, $e:expr) => {
        assert_eq!($x.unwrap_err().root_cause().to_string(), $e.to_string())
    };
}

pub use assert_error;

// Shorthand for an unchecked address.
#[macro_export]
macro_rules! addr {
    ($x:expr ) => {
        Addr::unchecked($x)
    };
}

pub use addr;
