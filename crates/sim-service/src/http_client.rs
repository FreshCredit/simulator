//! Canonical HTTP client provider for simulator service.

use std::sync::OnceLock;
use std::time::Duration;

#[must_use]
pub fn shared_client() -> &'static reqwest::Client {
    static SHARED: OnceLock<reqwest::Client> = OnceLock::new();
    SHARED.get_or_init(|| {
        reqwest::Client::builder()
            .user_agent("freshcredit-sim-service/0.1")
            .timeout(Duration::from_secs(30))
            .connect_timeout(Duration::from_secs(15))
            .build()
            .unwrap_or_default()
    })
}
