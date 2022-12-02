use cosmwasm_storage::PrefixedStorage;
use cw_multi_test::App;

/// Helps you add stuff to storage in your multitest tests
pub fn update_storage(
    app: &mut App,
    address: &[u8],
    function: &mut dyn FnMut(&mut PrefixedStorage) -> (),
) {
    app.init_modules(|_, _, storage| {
        let mut namespace = b"contract_data/".to_vec();
        namespace.extend_from_slice(address);

        let mut prefixed_storage = PrefixedStorage::multilevel(storage, &[b"wasm", &namespace]);

        function(&mut prefixed_storage);
    })
}
