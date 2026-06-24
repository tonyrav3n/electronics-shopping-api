use axum::{Form, extract::State, http::StatusCode};
use bigdecimal::BigDecimal;
use sqlx::PgPool;
use tracing::Instrument;
use uuid::Uuid;

#[derive(serde::Deserialize)]
pub struct FormData {
    name: String,
    brand: String,
    description: String,
    price: BigDecimal,
    stock: i32,
}

pub async fn add_product(
    State(pool): State<PgPool>,
    Form(form): Form<FormData>,
) -> StatusCode {
    let request_id = Uuid::new_v4();
    let request_span = tracing::info_span!(
        "Adding a new product",
        %request_id,
        product_name = %form.name,
        product_brand = %form.brand,
        product_description = %form.description,
        product_price = %form.price,
        product_stock = %form.stock,
    );
    let _request_span_guard = request_span.enter();
    let query_span = tracing::info_span!("Saving new product details in the database");
    match sqlx::query!(
        r#"
        INSERT INTO products (name, brand, description, price, stock)
        VALUES ($1, $2, $3, $4, $5)
        "#,
        form.name,
        form.brand,
        form.description,
        form.price,
        form.stock
    )
    .execute(&pool)
    .instrument(query_span)
    .await
    {
        Ok(_) => {
            tracing::info!(
                "request_id {} - New product details have been saved.",
                request_id
            );
            StatusCode::OK
        }
        Err(e) => {
            tracing::error!(
                "request_id {} - Failed to execute query: {:?}",
                request_id,
                e
            );
            StatusCode::INTERNAL_SERVER_ERROR
        }
    }
}
