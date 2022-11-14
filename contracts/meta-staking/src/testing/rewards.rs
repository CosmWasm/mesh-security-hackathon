use crate::error::ContractError;

fn test() -> Result<(), ContractError> {
    Err(ContractError::NoConsumer {})
}

fn test_cont() -> Result<(), ContractError> {
    test().ok().ok_or(ContractError::ConsumerAlreadyExists {})?;

    Ok(())
}

#[test]
fn testing() {
    let e = test_cont();
    println!("{:?}", e)
}
