mod utils;

// Test add / remove consumer
mod test_consumer;

const NATIVE_DENOM: &str = "NATIVE_DENOM";
const CONSUMER_1: &str = "consumer_1";
const CONSUMER_2: &str = "consumer_2";
const VALIDATOR: &str = "validator";

/// Assert the error is the error we
///
/// Result<AppResponse, Error> === ContractError::Example{}
macro_rules! assert_error {
    ($x:expr, $e:expr) => {
        assert_eq!($x.unwrap_err().root_cause().to_string(), $e.to_string())
    };
}

pub(crate) use assert_error;
