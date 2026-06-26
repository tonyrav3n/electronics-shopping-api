use electronics_shopping_api::configuration::get_configuration;
use electronics_shopping_api::startup::{AppState, app};
use sqlx::PgPool;
use tracing_subscriber::{EnvFilter, FmtSubscriber};

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let subscriber = FmtSubscriber::builder()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .finish();

    tracing::subscriber::set_global_default(subscriber).expect("Failed to set subscriber");

    let configuration = get_configuration().expect("Failed to read configuration.");
    let connection_pool = PgPool::connect(&configuration.database.connection_string())
        .await
        .expect("Failed to connect to Postgres");

    let state = AppState {
        db_pool: connection_pool,
        jwt_secret: configuration.jwt_secret,
    };

    let app = app(state);

    let address = format!("127.0.0.1:{}", configuration.application_port);
    let listener = tokio::net::TcpListener::bind(address).await?;
    tracing::info!("Started the API server on {}", listener.local_addr()?);

    axum::serve(listener, app).await
}
