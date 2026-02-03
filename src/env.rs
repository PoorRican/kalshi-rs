use url::Url;

/// REST API prefix (Kalshi Exchange REST v2).
pub const REST_PREFIX: &str = "/trade-api/v2";

/// WebSocket path (Kalshi Exchange WS v2) used for signing:
/// timestamp + "GET" + "/trade-api/ws/v2"
pub const WS_PATH: &str = "/trade-api/ws/v2";

const DEMO_URL: &str = "https://demo-api.kalshi.co/";
const LIVE_URL: &str = "https://api.elections.kalshi.com/";

#[derive(Debug, Clone)]
pub struct KalshiEnvironment {
    /// Origin only, e.g. https://demo-api.kalshi.co
    pub rest_origin: Url,
    /// Full WS URL, e.g. wss://demo-api.kalshi.co/trade-api/ws/v2
    pub ws_url: Url,
}

impl KalshiEnvironment {
    /// Demo environment.
    /// REST origin: https://demo-api.kalshi.co
    /// WS URL: wss://demo-api.kalshi.co/trade-api/ws/v2
    pub fn demo() -> Self {
        Self {
            rest_origin: Url::parse("https://demo-api.kalshi.co/").expect("valid demo REST origin"),
            ws_url: Url::parse("wss://demo-api.kalshi.co/trade-api/ws/v2")
                .expect("valid demo WS url"),
        }
    }

    /// Production environment.
    /// REST base in docs uses https://api.elections.kalshi.com/trade-api/v2/
    /// WS URL: wss://api.elections.kalshi.com/trade-api/ws/v2
    pub fn production() -> Self {
        Self {
            rest_origin: Url::parse("https://api.elections.kalshi.com/")
                .expect("valid prod REST origin"),
            ws_url: Url::parse("wss://api.elections.kalshi.com/trade-api/ws/v2")
                .expect("valid prod WS url"),
        }
    }
}


