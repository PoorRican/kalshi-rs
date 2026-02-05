/// Example of using authenticated REST endpoints:
/// - Gets balance
/// - Places an order

use kalshi::{
    BuySell, CreateOrderRequest, KalshiAuth, KalshiEnvironment, KalshiRestClient, MarketStatus,
    OrderType, YesNo, GetMarketsParams,
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

    let balance = client.get_balance().await?;
    println!(
        "balance: {} portfolio_value: {}",
        balance.balance, balance.portfolio_value
    );

    let resp = client
        .get_markets(GetMarketsParams {
            limit: Some(1),
            status: Some(MarketStatus::Open),
            ..Default::default()
        })
        .await?;

    let market = resp
        .markets
        .into_iter()
        .next()
        .ok_or_else(|| anyhow::anyhow!("No open markets found"))?;

    println!("market: {}", market.ticker);

    let order = CreateOrderRequest {
        ticker: market.ticker,
        side: YesNo::Yes,
        action: BuySell::Buy,
        count: Some(1),
        r#type: Some(OrderType::Limit),
        yes_price: Some(1),
        ..Default::default()
    };

    let created = client.create_order(order).await?;
    println!("order_id: {}", created.order.order_id);
    Ok(())
}
