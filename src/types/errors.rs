use serde::{Deserialize, Serialize};

use super::AnyJson;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ErrorResponse {
    #[serde(default)]
    pub code: Option<String>,
    #[serde(default)]
    pub message: Option<String>,
    #[serde(default)]
    pub details: Option<AnyJson>,
    #[serde(default)]
    pub service: Option<String>,
}
