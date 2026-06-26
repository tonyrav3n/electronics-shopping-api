use crate::startup::AppState;
use axum::{
    Form, Json,
    extract::{Path, Query, State},
    http::StatusCode,
};
use bigdecimal::BigDecimal;
use sqlx::{Postgres, QueryBuilder};
use tracing::Instrument;
use uuid::Uuid;
use validator::Validate;

#[derive(serde::Deserialize, Validate)]
pub struct ProductFormData {
    #[validate(length(max = 100))]
    name: String,
    #[validate(length(max = 50))]
    brand: String,
    #[validate(length(max = 500))]
    description: String,
    price: BigDecimal,
    stock: i32,
}

pub async fn add_product(
    State(state): State<AppState>,
    Form(form): Form<ProductFormData>,
) -> StatusCode {
    if form.validate().is_err() {
        return StatusCode::BAD_REQUEST;
    }

    let request_id = Uuid::new_v4();
    let request_span = tracing::info_span!(
        "Product creation requested",
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
    .execute(&state.db_pool)
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

#[derive(serde::Deserialize, Debug)]
pub struct SearchParams {
    name: Option<String>,
    brand: Option<String>,
    min_price: Option<BigDecimal>,
    max_price: Option<BigDecimal>,
}

#[derive(serde::Serialize, sqlx::FromRow)]
pub struct Product {
    pub id: Uuid,
    pub name: String,
    pub brand: String,
    pub description: Option<String>,
    pub price: BigDecimal,
    pub stock: i32,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

pub async fn search_products(
    State(state): State<AppState>,
    Query(params): Query<SearchParams>,
) -> Result<Json<Vec<Product>>, StatusCode> {
    let request_id = Uuid::new_v4();
    let request_span = tracing::info_span!(
        "Product search requested",
        %request_id,
        name = ?params.name,
        brand = ?params.brand,
        min_price = ?params.min_price,
        max_price = ?params.max_price,
    );
    let _request_span_guard = request_span.enter();

    let mut query: QueryBuilder<Postgres> = QueryBuilder::new(
        "SELECT id, name, brand, description, price, stock,
      created_at FROM products WHERE 1=1",
    );
    if let Some(name) = params.name {
        query.push(" AND name ILIKE ");
        query.push_bind(format!("%{}%", name));
    }

    if let Some(brand) = params.brand {
        query.push(" AND brand ILIKE ");
        query.push_bind(format!("%{}%", brand));
    }

    if let Some(min_price) = params.min_price {
        query.push(" AND price >= ");
        query.push_bind(min_price);
    }

    if let Some(max_price) = params.max_price {
        query.push(" AND price <= ");
        query.push_bind(max_price);
    }

    let query_span = tracing::info_span!("Querying products table", sql = query.sql().as_str());

    match query
        .build_query_as::<Product>()
        .fetch_all(&state.db_pool)
        .instrument(query_span)
        .await
    {
        Ok(products) => {
            tracing::info!(
                "request_id {} - Found {} products",
                request_id,
                products.len()
            );
            Ok(Json(products))
        }
        Err(e) => {
            tracing::error!(
                "request_id {} - Failed to execute query: {:?}",
                request_id,
                e
            );
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn get_product(
    State(state): State<AppState>,
    Path(product_id): Path<Uuid>,
) -> Result<Json<Product>, StatusCode> {
    let request_id = Uuid::new_v4();
    let request_span = tracing::info_span!(
        "Product details requested",
        %request_id,
        product_id = %product_id,
    );
    let _request_span_guard = request_span.enter();

    let query_span = tracing::info_span!(
        "Querying products table",
        sql = "SELECT id, name, brand, description, price, stock, created_at FROM products WHERE id = $1"
    );

    match sqlx::query_as!(
        Product,
        r#"
        SELECT id, name, brand, description, price, stock, created_at FROM products WHERE id = $1
        "#,
        product_id
    )
    .fetch_one(&state.db_pool)
    .instrument(query_span)
    .await
    {
        Ok(product) => {
            tracing::info!("request_id {} - Product found.", request_id);
            Ok(Json(product))
        }
        Err(sqlx::Error::RowNotFound) => {
            tracing::warn!("request_id {} - Product not found.", request_id);
            Err(StatusCode::NOT_FOUND)
        }
        Err(e) => {
            tracing::error!(
                "request_id {} - Failed to execute query: {:?}",
                request_id,
                e
            );
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}
