use electronics_shopping_api::configuration::{DatabaseSettings, get_configuration};
use electronics_shopping_api::startup::{AppState, app};
use sqlx::{Connection, PgConnection, PgPool};
use uuid::Uuid;

pub struct TestApp {
    pub address: String,
    #[allow(dead_code)]
    pub db_pool: PgPool,
}

pub async fn spawn_app() -> TestApp {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("Failed to bind random port");
    let port = listener.local_addr().unwrap().port();
    let address = format!("http://127.0.0.1:{}", port);

    let mut configuration = get_configuration().expect("Failed to read configuration");

    // Generate a random database name for isolation
    configuration.database.database_name = Uuid::new_v4().to_string();
    let connection_pool = configure_database(&configuration.database).await;

    let state = AppState {
        db_pool: connection_pool.clone(),
        jwt_secret: configuration.jwt_secret,
    };

    let app = app(state);

    tokio::spawn(async move {
        axum::serve(listener, app).await.expect("Server crashed");
    });

    TestApp {
        address,
        db_pool: connection_pool,
    }
}

pub async fn configure_database(config: &DatabaseSettings) -> PgPool {
    // 1. Connect without specifying a database
    let mut connection = PgConnection::connect(&config.connection_string_without_db())
        .await
        .expect("Failed to connect to Postgres");

    // 2. Create the random database
    let mut builder = sqlx::QueryBuilder::new("CREATE DATABASE \"");
    builder.push(&config.database_name);
    builder.push("\"");
    builder
        .build()
        .execute(&mut connection)
        .await
        .expect("Failed to create database.");

    // 3. Connect to the new database
    let connection_pool = PgPool::connect(&config.connection_string())
        .await
        .expect("Failed to connect to Postgres.");

    // 4. Run migrations on the isolated database
    sqlx::migrate!("./migrations")
        .run(&connection_pool)
        .await
        .expect("Failed to migrate the database");

    connection_pool
}
