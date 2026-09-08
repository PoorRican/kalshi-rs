use kalshi_fast::{
    CreateOrderGroupRequest, CreateQuoteRequest, CreateRFQRequest, KalshiAuth, KalshiEnvironment,
    KalshiRestClient, SubaccountQueryParams, UpdateOrderGroupLimitParams,
    UpdateOrderGroupLimitRequest,
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let key_id = std::env::var("KALSHI_KEY_ID")?;
    let private_key_path = std::env::var("KALSHI_PRIVATE_KEY_PATH")?;
    let market_ticker = match std::env::var("KALSHI_MARKET_TICKER") {
        Ok(value) => value,
        Err(_) => {
            eprintln!("set KALSHI_MARKET_TICKER to run this example");
            return Ok(());
        }
    };

    let auth = KalshiAuth::from_pem_file(key_id, private_key_path)?;
    let client = KalshiRestClient::new(KalshiEnvironment::demo()).with_auth(auth);

    let order_group = client
        .create_order_group(CreateOrderGroupRequest {
            contracts_limit_fp: Some("25.00".to_string()),
            ..Default::default()
        })
        .await?;
    println!("created order_group_id={}", order_group.order_group_id);

    client
        .update_order_group_limit(
            &order_group.order_group_id,
            UpdateOrderGroupLimitParams::default(),
            UpdateOrderGroupLimitRequest {
                contracts_limit_fp: Some("50.00".to_string()),
                ..Default::default()
            },
        )
        .await?;

    let rfq = client
        .create_rfq(CreateRFQRequest {
            market_ticker,
            contracts: None,
            contracts_fp: Some("10.00".to_string()),
            target_cost_centi_cents: None,
            target_cost_dollars: None,
            rest_remainder: true,
            replace_existing: None,
            subtrader_id: None,
            subaccount: None,
        })
        .await?;
    println!("created rfq_id={}", rfq.id);

    let quote = client
        .create_quote(CreateQuoteRequest {
            rfq_id: rfq.id,
            yes_bid: "0.5200".to_string(),
            no_bid: "0.4800".to_string(),
            rest_remainder: true,
            subaccount: None,
        })
        .await?;
    println!("created quote_id={}", quote.id);

    // Optional cleanup for the order group in demo environments.
    let _ = client
        .delete_order_group(
            &order_group.order_group_id,
            SubaccountQueryParams::default(),
        )
        .await;

    Ok(())
}
