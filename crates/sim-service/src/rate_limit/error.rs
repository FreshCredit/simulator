//! Error handling for rate limiting
//!
//! Provides custom error responses that follow the IETF `RateLimit` header draft
//! and include structured JSON error bodies.

use axum::body::Body;
use axum::http::StatusCode;
use axum::response::Response;
use tower_governor::GovernorError;

// TAG: surface=simulation owner=research-team rule=SIM-001
// TAG: surface=simulation owner=research-team rule=SIM-001
/// Custom error handler for rate limit violations
///
/// Converts `GovernorError` into a structured HTTP response with appropriate
/// status codes, headers, and JSON body.
///
/// # Response Format
///
/// When rate limit is exceeded:
/// ```json
/// {
///   "error": "rate_limit_exceeded",
///   "message": "Rate limit exceeded. Retry after 60 seconds.",
///   "retry_after": 60,
///   "type": "quota-exceeded"
/// }
/// ```
///
/// # Headers
///
/// - `Retry-After`: Seconds until the client can retry
/// - `RateLimit`: Current rate limit status (if available)
pub fn handle_rate_limit_error(error: GovernorError) -> Response<Body> {
    match error {
        GovernorError::TooManyRequests { wait_time, headers } => {
            let body = serde_json::json!({
                "error": "rate_limit_exceeded",
                "message": format!("Rate limit exceeded. Please retry after {} seconds.", wait_time),
                "retry_after": wait_time,
                "type": "quota-exceeded"
            });

            let mut response = Response::new(Body::from(body.to_string()));
            // TAG: surface=simulation owner=platform-team rule=GENERAL-001
            *response.status_mut() = StatusCode::TOO_MANY_REQUESTS;

            // Add standard headers
            response.headers_mut().insert(
                axum::http::header::CONTENT_TYPE,
                axum::http::HeaderValue::from_static("application/json"),
            );

            // Add rate limit headers from governor
            if let Some(header_map) = headers {
                response.headers_mut().extend(header_map);
            }

            tracing::warn!("Rate limit exceeded, retry after {}s", wait_time);

            response
        }

        GovernorError::UnableToExtractKey => {
            let body = serde_json::json!({
                "error": "unable_to_identify_client",
                "message": "Could not extract client identifier for rate limiting",
                "type": "client_identification_failed"
            });

            let mut response = Response::new(Body::from(body.to_string()));
            *response.status_mut() = StatusCode::BAD_REQUEST;

            response.headers_mut().insert(
                axum::http::header::CONTENT_TYPE,
                axum::http::HeaderValue::from_static("application/json"),
            );

            // TAG: surface=simulation owner=research-team rule=SIM-001
            tracing::warn!("Unable to extract rate limit key");

            response
        }

        GovernorError::Other { code, msg, headers } => {
            let body = msg.unwrap_or_else(|| "Rate limit error".to_string());
            let mut response = Response::new(Body::from(body));
            *response.status_mut() = code;

            if let Some(header_map) = headers {
                response.headers_mut().extend(header_map);
            }

            tracing::warn!("Other rate limit error: {:?}", code);

            response
        }
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Create a rate limit exceeded response with custom details
///
/// Use this when implementing custom rate limiting outside of `tower_governor`.
pub fn rate_limit_exceeded(
    retry_after_secs: u64,
    limit: Option<u32>,
    window: Option<&str>,
) -> Response<Body> {
    let mut body = serde_json::json!({
        "error": "rate_limit_exceeded",
        "message": format!("Rate limit exceeded. Please retry after {} seconds.", retry_after_secs),
        "retry_after": retry_after_secs,
        "type": "quota-exceeded"
    });

    if let Some(l) = limit {
        body["limit"] = serde_json::json!(l);
    }

    if let Some(w) = window {
        body["window"] = serde_json::json!(w);
    }

    let mut response = Response::new(Body::from(body.to_string()));
    *response.status_mut() = StatusCode::TOO_MANY_REQUESTS;

    response.headers_mut().insert(
        axum::http::header::CONTENT_TYPE,
        axum::http::HeaderValue::from_static("application/json"),
    );

    response.headers_mut().insert(
        axum::http::header::RETRY_AFTER,
        axum::http::HeaderValue::from_str(&retry_after_secs.to_string())
            .unwrap_or_else(|_| axum::http::HeaderValue::from_static("60")),
    );

    response
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Create a connection rate limit exceeded response
///
/// Used by the connection limiter for SSE/WebSocket endpoints.
pub fn connection_rate_limited(retry_after_secs: u64) -> Response<Body> {
    let body = serde_json::json!({
        "error": "connection_rate_limited",
        "message": format!("Too many connection attempts. Please retry after {} seconds.", retry_after_secs),
        "retry_after": retry_after_secs,
        "type": "connection-quota-exceeded"
    });

    let mut response = Response::new(Body::from(body.to_string()));
    *response.status_mut() = StatusCode::TOO_MANY_REQUESTS;

    response.headers_mut().insert(
        axum::http::header::CONTENT_TYPE,
        axum::http::HeaderValue::from_static("application/json"),
    );

    response.headers_mut().insert(
        axum::http::header::RETRY_AFTER,
        axum::http::HeaderValue::from_str(&retry_after_secs.to_string())
            .unwrap_or_else(|_| axum::http::HeaderValue::from_static("60")),
    );

    response
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_handle_too_many_requests() {
        let error = GovernorError::TooManyRequests {
            wait_time: 60,
            headers: None,
        };

        // TAG: surface=simulation owner=research-team rule=SIM-001
        let response = handle_rate_limit_error(error);

        assert_eq!(response.status(), StatusCode::TOO_MANY_REQUESTS);
    }

    #[test]
    fn test_rate_limit_exceeded_response() {
        let response = rate_limit_exceeded(60, Some(100), Some("60s"));

        assert_eq!(response.status(), StatusCode::TOO_MANY_REQUESTS);
        assert!(response
            .headers()
            .contains_key(axum::http::header::RETRY_AFTER));
    }

    #[test]
    fn test_connection_rate_limited_response() {
        let response = connection_rate_limited(30);

        assert_eq!(response.status(), StatusCode::TOO_MANY_REQUESTS);
        assert!(response
            .headers()
            .contains_key(axum::http::header::RETRY_AFTER));
    }
}
