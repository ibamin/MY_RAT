use axum::{extract::Request, http::StatusCode, middleware::Next, response::Response};

/// Operator auth middleware.
/// If `OPERATOR_TOKEN` env var is set, all requests must carry `Authorization: Bearer <token>`.
/// If the env var is not set, the middleware passes through (dev mode).
pub async fn require_operator_token(
    request: Request,
    next: Next,
) -> Result<Response, (StatusCode, String)> {
    let Ok(expected) = std::env::var("OPERATOR_TOKEN") else {
        return Ok(next.run(request).await);
    };

    let bearer = request
        .headers()
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "));

    if bearer == Some(expected.as_str()) {
        Ok(next.run(request).await)
    } else {
        Err((StatusCode::UNAUTHORIZED, "Unauthorized".to_string()))
    }
}
