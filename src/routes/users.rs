use axum::{
    Form, Json,
    extract::{Path, Query, State},
    http::StatusCode,
};
use bigdecimal::BigDecimal;
use sqlx::{PgPool, Postgres, QueryBuilder};
use tracing::Instrument;
use uuid::Uuid;
use validator::Validate;

#[derive(serde::Deserialize, Validate)]
pub struct UserFormData {
    #[validate(email)]
    email: String,
    #[validate(length(min = 8, max = 255))]
    password_hash: String,
    #[validate(length(min = 1, max = 50))]
    first_name: String,
    #[validate(length(min = 1, max = 50))]
    last_name: String,
    #[validate(length(min = 7, max = 20))]
    phone_number: String,
}

pub async fn add_user(State(pool): State<PgPool>, Form(form): Form<UserFormData>) -> StatusCode {
    if form.validate().is_err() {
        return StatusCode::BAD_REQUEST;
    }
    let request_id = Uuid::new_v4();
    let request_span = tracing::info_span!(
        "New user creation requested",
        %request_id,
        email = %form.email,
        password_hash = %form.password_hash,
        first_name = %form.first_name,
        last_name = %form.last_name,
        phone_number = %form.phone_number,
    );
    let _request_span_guard = request_span.enter();
    let query_span = tracing::info_span!("Saving new user details in the database");
    match sqlx::query!(
        r#"
        INSERT INTO users (email, password_hash, first_name, last_name, phone_number)
        VALUES ($1, $2, $3, $4, $5)
        "#,
        form.email,
        form.password_hash,
        form.first_name,
        form.last_name,
        form.phone_number
    )
    .execute(&pool)
    .instrument(query_span)
    .await
    {
        Ok(_) => {
            tracing::info!(
                "request_id {} - New user details have been saved.",
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
