mod helpers;

#[tokio::test]
async fn health_check_returns_200() {
    // Spin up the application in the background and get the address
    let app = helpers::spawn_app().await;
    let client = reqwest::Client::new();

    // Send a GET request to the health check endpoint
    let response = client
        .get(format!("{}/health_check", &app.address))
        .send()
        .await
        .expect("Failed to execute request.");

    // Assert that the response is successful (200 OK)
    assert!(response.status().is_success());
    assert_eq!(200, response.status().as_u16());
}
