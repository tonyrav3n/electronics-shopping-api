use crate::routes::{add_product, health_check};
use axum::{routing::{get, post}, Router};
use sqlx::PgPool;
use tower_http::trace::TraceLayer;

pub fn app(db_pool: PgPool) -> Router {
    Router::new()
        .route("/health_check", get(health_check))
        .route("/products", post(add_product))
        .layer(TraceLayer::new_for_http())
        .with_state(db_pool)
}
