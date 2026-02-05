use url::Url;

/// REST API prefix (Kalshi Exchange REST v2).
pub const REST_PREFIX: &str = "/trade-api/v2";

/// WebSocket path (Kalshi Exchange WS v2) used for signing:
/// timestamp + "GET" + "/trade-api/ws/v2"
pub const WS_PATH: &str = "/trade-api/ws/v2";

const DEMO_HOST: &str = "demo-api.kalshi.co";
const LIVE_HOST: &str = "api.elections.kalshi.com";

#[derive(Debug, Clone)]
pub struct KalshiEnvironment {
    /// Origin only, e.g. https://demo-api.kalshi.co (Url for reqwest compatibility)
    pub rest_origin: Url,
    /// Pre-computed WS URL string for direct use with tokio-tungstenite
    pub ws_url: String,
}

impl KalshiEnvironment {
    /// Demo environment.
    /// REST origin: https://demo-api.kalshi.co
    /// WS URL: wss://demo-api.kalshi.co/trade-api/ws/v2
    pub fn demo() -> Self {
        Self {
            rest_origin: Url::parse(&format!("https://{DEMO_HOST}/"))
                .expect("valid demo REST origin"),
            ws_url: format!("wss://{DEMO_HOST}{WS_PATH}"),
        }
    }

    /// Production environment.
    /// REST base in docs uses https://api.elections.kalshi.com/trade-api/v2/
    /// WS URL: wss://api.elections.kalshi.com/trade-api/ws/v2
    pub fn production() -> Self {
        Self {
            rest_origin: Url::parse(&format!("https://{LIVE_HOST}/"))
                .expect("valid prod REST origin"),
            ws_url: format!("wss://{LIVE_HOST}{WS_PATH}"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn demo_urls_are_valid() {
        let env = KalshiEnvironment::demo();
        assert!(env.rest_origin.as_str().starts_with("https://"));
        // Validate ws_url by parsing it
        let _ = Url::parse(&env.ws_url).expect("valid demo WS URL");
    }

    #[test]
    fn production_urls_are_valid() {
        let env = KalshiEnvironment::production();
        assert!(env.rest_origin.as_str().starts_with("https://"));
        let _ = Url::parse(&env.ws_url).expect("valid prod WS URL");
    }
}
