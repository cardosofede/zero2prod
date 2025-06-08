//! test/health_check.rs

use zero2prod::startup::run;
use std::net::TcpListener;
use sqlx::{Connection, PgConnection};
use zero2prod::configuration::get_configuration;

fn spawn_app() -> String {
    let listener = TcpListener::bind("127.0.0.1:0")
        .expect("Failed to bind to address");
    let port = listener.local_addr().unwrap().port();
    let server = run(listener).expect("Failed to bin address");
    let _ = tokio::spawn(server);
    format!("http://localhost:{}", port)
}

#[tokio::test]
async fn health_check_works() {
    let address = spawn_app();
    
    let client = reqwest::Client::new();
    
    let response = client
        .get(&format!("{}/health_check", &address))
        .send()
        .await
        .expect("Failed to send request");
    
    assert!(response.status().is_success(), "Expected a successful response, got: {}", response.status());
    assert_eq!(Some(0), response.content_length(), "Expected no content in the response, but got some");
}

#[tokio::test]
async fn subscribe_returns_a_200_for_valid_data() {
    let address = spawn_app();
    
    let configuration = get_configuration().expect("Failed to read configuration");
    let connection_string = configuration.database.connection_string();
    let mut connection = PgConnection::connect(&connection_string)
        .await
        .expect("Failed to connect to Postgres");
    
    let client = reqwest::Client::new();
    
    let body = "name=le%20guin&email=ursula_le_guin%40example.com";
    let response = client
        .post(&format!("{}/subscriptions", &address))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await
        .expect("Failed to send request");
    
    assert_eq!(200, response.status().as_u16(), "Expected a 200 OK response, got: {}", response.status());
    
    let saved = sqlx::query!("SELECT email, name FROM subscriptions")
        .fetch_one(&mut connection)
        .await
        .expect("Failed to fetch saved subscription");
    assert_eq!(saved.email, "ursula_le_guin@gmail.com");
    assert_eq!(saved.name, "le guin", "Expected name to be 'le guin', got: {}", saved.name);
}

#[tokio::test]
async fn subscribe_returns_a_400_when_data_is_missing() {
    let address = spawn_app();
    
    let client = reqwest::Client::new();
    let test_cases = vec![
        ("name=le%20guin", "missing the email"),
        ("email=ursula_le_guin%40example.com", "missing the name"),
        ("", "missing both name and email"),
    ];
    
    for (invalid_body, error_message) in test_cases {
    let response = client
            .post(&format!("{}/subscriptions", &address))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(invalid_body)
            .send()
            .await
            .expect("Failed to send request");
        
        assert_eq!(400, response.status().as_u16(), "Expected a 400 Bad Request response for {}, got: {}", error_message, response.status());
    }
}