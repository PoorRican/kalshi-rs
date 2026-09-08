#![cfg(feature = "live-tests")]

mod common;

use kalshi_fast::{
    CreateOrderGroupRequest, KalshiRestClient, SubaccountQueryParams, UpdateOrderGroupLimitParams,
    UpdateOrderGroupLimitRequest,
};
use std::time::Duration;

const LIFECYCLE_TIMEOUT: Duration = Duration::from_secs(30);

#[tokio::test]
async fn test_order_group_lifecycle() {
    common::load_env();
    let auth = common::load_auth();
    let client = common::demo_auth_client(auth);

    // 1. Create an order group
    let create_resp = tokio::time::timeout(LIFECYCLE_TIMEOUT, async {
        client
            .create_order_group(CreateOrderGroupRequest {
                contracts_limit: Some(100),
                ..Default::default()
            })
            .await
    })
    .await
    .expect("timeout")
    .expect("create_order_group failed");

    let group_id = create_resp.order_group_id.clone();
    assert!(!group_id.is_empty());

    // 2. Get order group and verify
    let get_resp = tokio::time::timeout(common::TEST_TIMEOUT, async {
        client
            .get_order_group(&group_id, SubaccountQueryParams::default())
            .await
    })
    .await
    .expect("timeout")
    .expect("get_order_group failed");

    assert_eq!(get_resp.contracts_limit, Some(100));

    // 3. Update the order group limit
    let _update_resp = tokio::time::timeout(common::TEST_TIMEOUT, async {
        client
            .update_order_group_limit(
                &group_id,
                UpdateOrderGroupLimitParams::default(),
                UpdateOrderGroupLimitRequest {
                    contracts_limit: Some(200),
                    ..Default::default()
                },
            )
            .await
    })
    .await
    .expect("timeout")
    .expect("update_order_group_limit failed");

    // Verify the update
    let get_resp2 = tokio::time::timeout(common::TEST_TIMEOUT, async {
        client
            .get_order_group(&group_id, SubaccountQueryParams::default())
            .await
    })
    .await
    .expect("timeout")
    .expect("get_order_group after update failed");

    assert_eq!(get_resp2.contracts_limit, Some(200));

    // 4. Reset order group
    let _reset_resp = tokio::time::timeout(common::TEST_TIMEOUT, async {
        client
            .reset_order_group(&group_id, SubaccountQueryParams::default())
            .await
    })
    .await
    .expect("timeout")
    .expect("reset_order_group failed");

    // 5. Trigger order group
    let _trigger_resp = tokio::time::timeout(common::TEST_TIMEOUT, async {
        client
            .trigger_order_group(&group_id, SubaccountQueryParams::default())
            .await
    })
    .await
    .expect("timeout")
    .expect("trigger_order_group failed");

    // 6. Delete order group (cleanup)
    let _delete_resp = tokio::time::timeout(common::TEST_TIMEOUT, async {
        client
            .delete_order_group(&group_id, SubaccountQueryParams::default())
            .await
    })
    .await
    .expect("timeout")
    .expect("delete_order_group failed");
}
