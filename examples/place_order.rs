/// Example of using authenticated REST endpoints:
/// - Gets balance
/// - Places an order via the V2 event-order endpoint
use kalshi_fast::{
    BookSide, CreateOrderV2Request, GetBalanceParams, GetMarketsParams, KalshiAuth,
    KalshiEnvironment, KalshiRestClient, MarketStatusQuery, SelfTradePreventionType, TimeInForce,
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();

    let env = KalshiEnvironment::demo();
    let auth = KalshiAuth::from_pem_file(
        std::env::var("KALSHI_KEY_ID")?,
        std::env::var("KALSHI_PRIVATE_KEY_PATH")?,
    )?;
    let client = KalshiRestClient::new(env).with_auth(auth);

    let balance = client.get_balance(GetBalanceParams::default()).await?;
    println!(
        "balance: {} portfolio_value: {}",
        balance.balance, balance.portfolio_value
    );

    let resp = client
        .get_markets(GetMarketsParams {
            limit: Some(1),
            status: Some(MarketStatusQuery::Open),
            ..Default::default()
        })
        .await?;

    let market = resp
        .markets
        .into_iter()
        .next()
        .ok_or_else(|| anyhow::anyhow!("No open markets found"))?;

    println!("market: {}", market.ticker);

    let order = CreateOrderV2Request {
        ticker: market.ticker,
        side: BookSide::Bid, // bid == yes
        count: "1".to_string(),
        price: "0.01".to_string(),
        time_in_force: TimeInForce::GoodTillCanceled,
        self_trade_prevention_type: SelfTradePreventionType::TakerAtCross,
        client_order_id: None,
        expiration_time: None,
        post_only: None,
        cancel_order_on_pause: None,
        reduce_only: None,
        subaccount: None,
        order_group_id: None,
        exchange_index: None,
    };

    let created = client.create_order_v2(order).await?;
    println!("order_id: {}", created.order_id);
    Ok(())
}
