use axum::{
    body::Body,
    http::Request,
    middleware::Next,
    response::Response,
};
use uuid::Uuid;

const X_CORRELATION_ID: &str = "x-correlation-id";

pub async fn correlation_id(
    req: Request<Body>,
    next: Next,
) -> Response {
    let correlation_id = req
        .headers()
        .get(X_CORRELATION_ID)
        .and_then(|h| h.to_str().ok())
        .map(|s| s.to_string())
        .unwrap_or_else(|| Uuid::new_v4().to_string());

    tracing::Span::current().record("correlation_id", &correlation_id);
    
    let mut response = next.run(req).await;
    
    response.headers_mut().insert(
        X_CORRELATION_ID,
        correlation_id.parse().unwrap(),
    );
    
    response
}