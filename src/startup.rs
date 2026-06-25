use crate::routes::{add_product, add_user, get_product, health_check, search_products};
use axum::{
    Router,
    routing::{get, post},
};
use sqlx::PgPool;
use tower_http::trace::TraceLayer;

pub fn app(db_pool: PgPool) -> Router {
    Router::new()
        .route("/health_check", get(health_check))
        .route("/products", get(search_products).post(add_product))
        .route("/product/:id", get(get_product))
        .route("/user/register", post(add_user))
        .layer(TraceLayer::new_for_http())
        .with_state(db_pool)
}
