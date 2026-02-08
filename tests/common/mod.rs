use kalshi_fast::{KalshiAuth, KalshiEnvironment};
use std::time::Duration;

pub const TEST_TIMEOUT: Duration = Duration::from_secs(10);

#[allow(dead_code)]
pub fn load_env() {
    dotenvy::from_filename(".env.test").ok();
}

// integration tests cannot import from unit tests
#[allow(dead_code)]
pub fn load_auth() -> KalshiAuth {
    dotenvy::from_filename(".env.test").ok();

    let key_id = std::env::var("KALSHI_KEY_ID").expect("KALSHI_KEY_ID required");

    if let Ok(pem_content) = std::env::var("KALSHI_PRIVATE_KEY") {
        let pem_content = pem_content.replace("\\n", "\n");
        KalshiAuth::from_pem_str(key_id, &pem_content).expect("load auth from KALSHI_PRIVATE_KEY")
    } else {
        let pem_path = std::env::var("KALSHI_PRIVATE_KEY_PATH")
            .expect("KALSHI_PRIVATE_KEY or KALSHI_PRIVATE_KEY_PATH required");
        KalshiAuth::from_pem_file(key_id, pem_path).expect("load auth from KALSHI_PRIVATE_KEY_PATH")
    }
}

pub fn demo_env() -> KalshiEnvironment {
    KalshiEnvironment::demo()
}
