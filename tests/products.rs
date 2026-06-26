mod helpers;

#[tokio::test]
async fn add_product_returns_200_for_valid_form_data() {
    // Spin up the application and create an HTTP client
    let app = helpers::spawn_app().await;
    let client = reqwest::Client::new();

    // Prepare valid url-encoded form data for a new product
    let body = "name=PlayStation%205&brand=Sony&description=Gaming%20Console&price=499&stock=10";

    // Send a POST request to add the product
    let response = client
        .post(format!("{}/products", &app.address))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await
        .expect("Failed to execute request.");

    // Assert that the product was successfully added
    assert_eq!(200, response.status().as_u16());
}

#[tokio::test]
async fn search_products_works() {
    let app = helpers::spawn_app().await;
    let client = reqwest::Client::new();

    // 1. Seed a product into the isolated test database
    let body = "name=PlayStation%205&brand=Sony&description=Gaming%20Console&price=499&stock=10";
    client
        .post(format!("{}/products", &app.address))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await
        .expect("Failed to execute POST request.");

    // 2. Test a search that SHOULD find the product
    let response = client
        .get(format!("{}/products?name=playstation", &app.address))
        .send()
        .await
        .expect("Failed to execute GET request.");

    assert_eq!(200, response.status().as_u16());

    // Parse the JSON response
    let json: Vec<serde_json::Value> = response.json().await.expect("Failed to parse JSON");
    assert_eq!(json.len(), 1); // We should find exactly 1 product
    assert_eq!(json[0]["name"], "PlayStation 5");
    assert_eq!(json[0]["brand"], "Sony");

    // 3. Test a search that should NOT find the product
    let empty_response = client
        .get(format!("{}/products?brand=microsoft", &app.address))
        .send()
        .await
        .expect("Failed to execute GET request.");

    let empty_json: Vec<serde_json::Value> =
        empty_response.json().await.expect("Failed to parse JSON");
    assert_eq!(empty_json.len(), 0); // Should be empty!
}

#[tokio::test]
async fn get_product_returns_200_for_existing_product() {
    let app = helpers::spawn_app().await;
    let client = reqwest::Client::new();

    // 1. Seed a product
    let body = "name=PlayStation%205&brand=Sony&description=Gaming%20Console&price=499&stock=10";
    client
        .post(format!("{}/products", &app.address))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await
        .expect("Failed to execute POST request.");

    // Retrieve the generated product ID via search
    let search_res = client
        .get(format!("{}/products?name=playstation", &app.address))
        .send()
        .await
        .expect("Failed to execute search GET request.");
    let json: Vec<serde_json::Value> = search_res.json().await.expect("Failed to parse JSON");
    let product_id = json[0]["id"].as_str().unwrap();

    // 2. Fetch the product directly using get_product
    let get_res = client
        .get(format!("{}/product/{}", &app.address, product_id))
        .send()
        .await
        .expect("Failed to execute GET product request.");

    assert_eq!(200, get_res.status().as_u16());
    let get_json: serde_json::Value = get_res.json().await.expect("Failed to parse JSON");
    assert_eq!(get_json["name"], "PlayStation 5");
    assert_eq!(get_json["brand"], "Sony");
}

#[tokio::test]
async fn get_product_returns_404_for_missing_product() {
    let app = helpers::spawn_app().await;
    let client = reqwest::Client::new();
    let random_id = uuid::Uuid::new_v4();

    let response = client
        .get(format!("{}/product/{}", &app.address, random_id))
        .send()
        .await
        .expect("Failed to execute request.");

    assert_eq!(404, response.status().as_u16());
}

#[tokio::test]
async fn add_product_returns_422_when_data_is_missing() {
    let app = helpers::spawn_app().await;
    let client = reqwest::Client::new();
    let test_cases = vec![
        ("name=TestProduct&brand=TestBrand&price=99.99", "missing stock and description"),
        ("name=TestProduct&brand=TestBrand&description=TestDesc&stock=10", "missing price"),
    ];

    for (invalid_body, error_message) in test_cases {
        let response = client
            .post(format!("{}/products", &app.address))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(invalid_body)
            .send()
            .await
            .expect("Failed to execute request.");

        assert_eq!(
            422,
            response.status().as_u16(),
            "The API did not fail with 422 Unprocessable Entity when the payload was {}.",
            error_message
        );
    }
}

#[tokio::test]
async fn add_product_returns_400_when_fields_are_present_but_invalid() {
    let app = helpers::spawn_app().await;
    let client = reqwest::Client::new();
    let too_long_name = "A".repeat(101);
    let invalid_body = format!("name={}&brand=TestBrand&description=TestDesc&price=99.99&stock=10", too_long_name);

    let response = client
        .post(format!("{}/products", &app.address))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(invalid_body)
        .send()
        .await
        .expect("Failed to execute request.");

    assert_eq!(
        400,
        response.status().as_u16(),
        "The API did not fail with 400 Bad Request when the product name was too long."
    );
}
