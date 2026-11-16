// TAG: surface=blockchain owner=blockchain-team rule=BC-001
// POT-005: CORS layer builder helper
use tower_http::cors::{AllowOrigin, CorsLayer};

/// CORS layer configuration - environment-aware
/// P1-SECURITY: Restricted origins to prevent CSRF attacks
pub fn cors_layer() -> CorsLayer {
    // Allow specific origins based on environment
    // P1-SECURITY: Never use AllowOrigin::any() - even in development
    let allowed_origins = if std::env::var("PRODUCTION").is_ok() {
        let origins: Vec<_> = [
            "https://freshcredit.com",
            "https://app.freshcredit.com",
            "https://testnet.freshcredit.com",
        ]
        .iter()
        .filter_map(|o| o.parse().ok())
        .collect();
        AllowOrigin::list(origins)
    } else {
        // Development: restrict to known local origins
        let origins: Vec<_> = [
            "http://localhost:3000",
            "http://localhost:3001",
            "http://localhost:5173",
            "http://127.0.0.1:3000",
            "http://127.0.0.1:3001",
            "http://127.0.0.1:5173",
        ]
        .iter()
        .filter_map(|o| o.parse().ok())
        .collect();
        AllowOrigin::list(origins)
    };

    CorsLayer::new()
        .allow_origin(allowed_origins)
        .allow_methods([
            http::Method::GET,
            http::Method::POST,
            http::Method::PUT,
            http::Method::DELETE,
        ])
        .allow_headers([
            http::header::CONTENT_TYPE,
            http::header::AUTHORIZATION,
            http::header::ACCEPT,
            http::HeaderName::from_static("x-api-key"),
        ])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cors_layer_does_not_panic() {
        let layer = cors_layer();
        // Verify it returns a CorsLayer (type check at compile time)
        let _ = layer;
    }
    // TAG: surface=blockchain owner=blockchain-team rule=BC-001
}
