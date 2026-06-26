use crate::routes::{
    add_product, add_user, get_product, health_check, login_user, search_products,
};
use axum::{
    Router,
    routing::{get, post},
};
use sqlx::PgPool;
use tower_http::trace::TraceLayer;

#[derive(Clone)]
pub struct AppState {
    pub db_pool: PgPool,
    pub jwt_secret: String,
}

pub fn app(state: AppState) -> Router {
    Router::new()
        .route("/health_check", get(health_check))
        .route("/products", get(search_products).post(add_product))
        .route("/product/:id", get(get_product))
        .route("/user/register", post(add_user))
        .route("/user/login", post(login_user))
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}
