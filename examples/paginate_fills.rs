/// Paginate through authenticated fill history.
///
/// Demonstrates authenticated pager usage with statistics accumulation.
/// Requires KALSHI_KEY_ID and KALSHI_PRIVATE_KEY_PATH environment variables.
use kalshi::{GetFillsParams, KalshiAuth, KalshiEnvironment, KalshiRestClient};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();

    let auth = KalshiAuth::from_pem_file(
        std::env::var("KALSHI_KEY_ID")?,
        std::env::var("KALSHI_PRIVATE_KEY_PATH")?,
    )?;
    let client = KalshiRestClient::new(KalshiEnvironment::production()).with_auth(auth);

    let mut pager = client.fills_pager(GetFillsParams::default());
    let mut total_fills = 0;

    while let Some(fills) = pager.next_page().await? {
        total_fills += fills.len();
        for fill in fills {
            println!("{:?}", fill.ticker);
        }
    }

    println!("Total fills: {}", total_fills);
    Ok(())
}
