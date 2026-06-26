mod helpers;

#[tokio::test]
async fn add_user_returns_a_200_for_valid_form_data() {
    let app = helpers::spawn_app().await;
    let client = reqwest::Client::new();

    let body = "email=test%40example.com&password=securepassword123&first_name=John&last_name=Doe&phone_number=1234567890";

    let response = client
        .post(format!("{}/user/register", &app.address))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await
        .expect("Failed to execute request.");

    assert_eq!(200, response.status().as_u16());

    let saved =
        sqlx::query!("SELECT email, password_hash, first_name, last_name, phone_number FROM users")
            .fetch_one(&app.db_pool)
            .await
            .expect("Failed to fetch saved user.");

    assert_eq!(saved.email, "test@example.com");
    assert_eq!(saved.first_name, Some("John".to_string()));
    assert_eq!(saved.last_name, Some("Doe".to_string()));
    assert_eq!(saved.phone_number, Some("1234567890".to_string()));
    // Verify it actually hashed the password properly
    assert_ne!(saved.password_hash, "securepassword123");
    assert!(saved.password_hash.starts_with("$argon2"));
}

#[tokio::test]
async fn add_user_returns_a_400_when_data_is_missing() {
    let app = helpers::spawn_app().await;
    let client = reqwest::Client::new();

    let test_cases = vec![
        (
            "email=test%40example.com&password=securepassword&first_name=John&last_name=Doe",
            "missing the phone number",
        ),
        (
            "email=test%40example.com&password=securepassword&first_name=John",
            "missing the last name",
        ),
        (
            "email=test%40example.com&password=securepassword",
            "missing the first name",
        ),
        ("email=test%40example.com", "missing the password"),
        ("", "missing all fields"),
    ];

    for (invalid_body, error_message) in test_cases {
        let response = client
            .post(format!("{}/user/register", &app.address))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(invalid_body)
            .send()
            .await
            .expect("Failed to execute request.");

        assert_eq!(
            422, // Unprocessable Entity
            response.status().as_u16(),
            "The API did not fail with 422 Unprocessable Entity when the payload was {}.",
            error_message
        );
    }
}

#[tokio::test]
async fn add_user_returns_a_400_when_fields_are_present_but_invalid() {
    let app = helpers::spawn_app().await;
    let client = reqwest::Client::new();

    let test_cases = vec![
        (
            "email=not-an-email&password=securepassword&first_name=John&last_name=Doe&phone_number=1234567",
            "invalid email",
        ),
        (
            "email=test%40example.com&password=short&first_name=John&last_name=Doe&phone_number=1234567",
            "password too short (< 8 chars)",
        ),
        (
            "email=test%40example.com&password=securepassword&first_name=&last_name=Doe&phone_number=1234567",
            "empty first name",
        ),
    ];

    for (invalid_body, error_message) in test_cases {
        let response = client
            .post(format!("{}/user/register", &app.address))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(invalid_body)
            .send()
            .await
            .expect("Failed to execute request.");

        assert_eq!(
            400, // Bad Request
            response.status().as_u16(),
            "The API did not fail with 400 Bad Request when the payload was {}.",
            error_message
        );
    }
}

#[tokio::test]
async fn login_returns_200_for_valid_credentials() {
    let app = helpers::spawn_app().await;
    let client = reqwest::Client::new();
    
    // 1. Register a user
    client
        .post(format!("{}/user/register", &app.address))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body("email=login%40example.com&password=securepassword123&first_name=John&last_name=Doe&phone_number=1234567890")
        .send()
        .await
        .expect("Failed to execute request.");

    // 2. Login with same user
    let response = client
        .post(format!("{}/user/login", &app.address))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body("email=login%40example.com&password=securepassword123")
        .send()
        .await
        .expect("Failed to execute login request.");

    assert_eq!(200, response.status().as_u16());
}

#[tokio::test]
async fn login_returns_401_for_invalid_password() {
    let app = helpers::spawn_app().await;
    let client = reqwest::Client::new();
    
    client
        .post(format!("{}/user/register", &app.address))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body("email=wrongpass%40example.com&password=securepassword123&first_name=John&last_name=Doe&phone_number=1234567890")
        .send()
        .await
        .expect("Failed to execute request.");

    let response = client
        .post(format!("{}/user/login", &app.address))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body("email=wrongpass%40example.com&password=WRONGpassword")
        .send()
        .await
        .expect("Failed to execute login request.");

    assert_eq!(401, response.status().as_u16());
}

#[tokio::test]
async fn login_returns_401_for_missing_email() {
    let app = helpers::spawn_app().await;
    let client = reqwest::Client::new();
    
    let response = client
        .post(format!("{}/user/login", &app.address))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body("email=doesnotexist%40example.com&password=securepassword123")
        .send()
        .await
        .expect("Failed to execute login request.");

    assert_eq!(401, response.status().as_u16());
}
