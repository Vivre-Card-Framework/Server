use axum::{routing, Router};
use tower_http::trace::TraceLayer;

pub fn get() -> Router {
    Router::new()
        .route(
            "/",
            routing::get(async || "Hello Vivre Card Framework Server!"),
        )
        .layer(TraceLayer::new_for_http())
}
