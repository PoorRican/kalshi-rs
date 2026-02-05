/// Cursor checkpointing for resumable pagination jobs.
///
/// Demonstrates saving and restoring pagination state across sessions.
/// Run multiple times to see it resume from the last checkpoint.
use kalshi::{GetMarketsParams, KalshiEnvironment, KalshiRestClient, MarketStatus};
use std::fs;

const CURSOR_FILE: &str = "/tmp/market_scan_cursor.txt";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let client = KalshiRestClient::new(KalshiEnvironment::production());

    // Resume from saved cursor if exists
    let start_cursor = fs::read_to_string(CURSOR_FILE).ok();
    if start_cursor.is_some() {
        println!("Resuming from saved cursor...");
    }

    let mut pager = client.markets_pager(GetMarketsParams {
        cursor: start_cursor,
        status: Some(MarketStatus::Open),
        limit: Some(100),
        ..Default::default()
    });

    let mut processed = 0;
    while let Some(markets) = pager.next_page().await? {
        for market in &markets {
            println!("{}", market.ticker);
            processed += 1;
        }

        // Checkpoint cursor after each page
        if let Some(cursor) = pager.current_cursor() {
            fs::write(CURSOR_FILE, cursor)?;
        }

        if processed >= 500 {
            println!(
                "Session limit reached ({} markets) - resume later",
                processed
            );
            return Ok(());
        }
    }

    // Done - clean up cursor file
    let _ = fs::remove_file(CURSOR_FILE);
    println!("Scan complete: {} markets", processed);
    Ok(())
}
