use crate::startup::AppState;
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use axum::{Form, extract::State, http::StatusCode};
use tracing::Instrument;
use uuid::Uuid;
use validator::Validate;

#[derive(serde::Deserialize, Validate)]
pub struct UserRegisterFormData {
    #[validate(email)]
    email: String,
    #[validate(length(min = 8, max = 255))]
    password: String,
    #[validate(length(min = 1, max = 50))]
    first_name: String,
    #[validate(length(min = 1, max = 50))]
    last_name: String,
    #[validate(length(min = 7, max = 20))]
    phone_number: String,
}

pub async fn add_user(
    State(state): State<AppState>,
    Form(form): Form<UserRegisterFormData>,
) -> StatusCode {
    if form.validate().is_err() {
        return StatusCode::BAD_REQUEST;
    }

    let salt =
        argon2::password_hash::SaltString::generate(&mut argon2::password_hash::rand_core::OsRng);
    let argon2 = Argon2::default();
    let hashed_password = argon2
        .hash_password(form.password.as_bytes(), &salt)
        .unwrap()
        .to_string();

    let request_id = Uuid::new_v4();
    let request_span = tracing::info_span!(
        "New user creation requested",
        %request_id,
        email = %form.email,
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
        hashed_password,
        form.first_name,
        form.last_name,
        form.phone_number
    )
    .execute(&state.db_pool)
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

#[derive(serde::Deserialize, Validate)]
pub struct UserLoginFormData {
    #[validate(email)]
    email: String,
    #[validate(length(min = 8, max = 255))]
    password: String,
}

pub async fn login_user(
    State(state): State<AppState>,
    Form(form): Form<UserLoginFormData>,
) -> StatusCode {
    if form.validate().is_err() {
        return StatusCode::BAD_REQUEST;
    }

    let request_id = Uuid::new_v4();
    let request_span = tracing::info_span!(
        "User login requested",
        %request_id,
        email = %form.email,

    );
    let _request_span_guard = request_span.enter();
    let query_span = tracing::info_span!("Fetching password hash from database");
    match sqlx::query!(
        r#"
        SELECT password_hash FROM users WHERE email = $1
        "#,
        form.email,
    )
    .fetch_one(&state.db_pool)
    .instrument(query_span)
    .await
    {
        Ok(record) => {
            let parsed_hash = match PasswordHash::new(&record.password_hash) {
                Ok(hash) => hash,
                Err(e) => {
                    tracing::error!(
                        "request_id {} - Failed to parse database hash: {:?}",
                        request_id,
                        e
                    );
                    return StatusCode::INTERNAL_SERVER_ERROR;
                }
            };

            if Argon2::default()
                .verify_password(form.password.as_bytes(), &parsed_hash)
                .is_ok()
            {
                tracing::info!("request_id {} - User successfully logged in.", request_id);
                StatusCode::OK
            } else {
                tracing::warn!("request_id {} - Invalid password.", request_id);
                StatusCode::UNAUTHORIZED
            }
        }
        Err(sqlx::Error::RowNotFound) => {
            tracing::warn!("request_id {} - Email not found.", request_id);
            StatusCode::UNAUTHORIZED
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
