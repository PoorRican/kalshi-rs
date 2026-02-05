use kalshi_fast::{KalshiAuth, KalshiEnvironment};
use std::time::Duration;

pub const TEST_TIMEOUT: Duration = Duration::from_secs(10);

#[allow(dead_code)]
pub fn load_env() {
    dotenvy::from_filename(".env.test").ok();
}

#[allow(dead_code)]
pub fn load_auth() -> KalshiAuth {
    let key_id = std::env::var("KALSHI_KEY_ID").expect("KALSHI_KEY_ID required");
    let pem_path =
        std::env::var("KALSHI_PRIVATE_KEY_PATH").expect("KALSHI_PRIVATE_KEY_PATH required");
    KalshiAuth::from_pem_file(key_id, pem_path).expect("Failed to load auth")
}

pub fn demo_env() -> KalshiEnvironment {
    KalshiEnvironment::demo()
}
