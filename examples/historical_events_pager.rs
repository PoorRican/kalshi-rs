/// Paginate through settled events with custom termination logic.
///
/// Uses CursorPager for page-by-page control, stopping when all events in a
/// batch closed before December 2025. The API has no `max_close_ts` filter,
/// so we check `close_ts` client-side.

use chrono::{TimeZone, Utc};
use kalshi::{EventStatus, GetEventsParams, KalshiEnvironment, KalshiRestClient};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let client = KalshiRestClient::new(KalshiEnvironment::production());

    // December 1, 2025 00:00:00 UTC
    let cutoff_ts = Utc.with_ymd_and_hms(2025, 12, 1, 0, 0, 0).unwrap().timestamp();

    let mut pager = client.events_pager(GetEventsParams {
        status: Some(EventStatus::Settled),
        limit: Some(200),
        ..Default::default()
    });

    while let Some(events) = pager.next_page().await? {
        let all_before_cutoff = events
            .iter()
            .all(|e| e.close_ts.map_or(false, |ts| ts < cutoff_ts));

        for event in &events {
            println!("{} | close_ts: {:?}", event.event_ticker, event.close_ts);
        }

        if all_before_cutoff {
            println!("All events closed before Dec 2025 - stopping");
            break;
        }
    }

    Ok(())
}
