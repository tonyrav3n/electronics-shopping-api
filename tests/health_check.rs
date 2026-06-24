use electronics_shopping_api::configuration::get_configuration;
use sqlx::PgPool;

pub struct TestApp {
    pub address: String,
    pub db_pool: PgPool,
}

async fn spawn_app() -> TestApp {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.expect("Failed to bind random port");
    let port = listener.local_addr().unwrap().port();
    let address = format!("http://127.0.0.1:{}", port);

    let configuration = get_configuration().expect("Failed to read configuration");
    let connection_pool = PgPool::connect(&configuration.database.connection_string())
        .await
        .expect("Failed to connect to Postgres.");

    let app = electronics_shopping_api::startup::app(connection_pool.clone());

    tokio::spawn(async move {
        axum::serve(listener, app).await.expect("Server crashed");
    });

    TestApp {
        address,
        db_pool: connection_pool,
    }
}

#[tokio::test]
async fn health_check_returns_200() {
    let app = spawn_app().await;
    let client = reqwest::Client::new();

    let response = client
        .get(format!("{}/health_check", &app.address))
        .send()
        .await
        .expect("Failed to execute request.");

    assert!(response.status().is_success());
    assert_eq!(200, response.status().as_u16());
}

#[tokio::test]
async fn add_product_returns_200_for_valid_form_data() {
    let app = spawn_app().await;
    let client = reqwest::Client::new();

    let body = "name=PlayStation%205&brand=Sony&description=Gaming%20Console&price=499&stock=10";

    let response = client
        .post(format!("{}/products", &app.address))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await
        .expect("Failed to execute request.");

    assert_eq!(200, response.status().as_u16());
}
