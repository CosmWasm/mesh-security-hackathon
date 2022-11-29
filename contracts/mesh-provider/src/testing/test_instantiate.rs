use mesh_testing::{
    constants::{CONNECTION_ID, CREATOR_ADDR},
    msgs::SlasherConfigResponse,
};

use crate::{
    msg::ConsumerInfo,
    testing::utils::queries::{query_provider_config, query_slasher_config},
};

use super::utils::setup::setup_app_with_contract;

#[test]
fn test_instantiate() {
    let (app, mesh_provider_addr) = setup_app_with_contract();

    let provider_config = query_provider_config(&app, mesh_provider_addr.as_str()).unwrap();
    let mesh_slasher_addr = provider_config.slasher.clone().unwrap();

    assert!(provider_config.slasher.is_some());
    assert_eq!(
        provider_config.consumer,
        ConsumerInfo {
            connection_id: CONNECTION_ID.to_string()
        }
    );

    let slasher_config = query_slasher_config(&app, mesh_slasher_addr.as_str()).unwrap();

    assert_eq!(
        slasher_config,
        SlasherConfigResponse {
            owner: CREATOR_ADDR.to_string(),
            slashee: mesh_provider_addr.to_string()
        }
    );
    // println!("{:?}", config)
}
