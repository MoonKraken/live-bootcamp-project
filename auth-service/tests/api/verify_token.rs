use auth_service::{domain::email::Email, utils::auth::generate_auth_token};
use serde_json::json;

use crate::helpers::{get_random_email, TestApp};

#[tokio::test]
async fn should_return_422_if_malformed_input() {
    let app = TestApp::new().await;
    let response = app.post_verify_token(&"woeifj".to_string()).await;

    assert_eq!(response.status().as_u16(), 422);
}

#[tokio::test]
async fn should_return_200_valid_token() {
    let app = TestApp::new().await;

    let email = Email::parse(get_random_email()).expect("email should be parseable");
    let valid_token = generate_auth_token(&email).expect("should generate token");

    let response = app.post_verify_token(&json!({
        "token": valid_token,
    })).await;

    assert_eq!(response.status().as_u16(), 200);
}

#[tokio::test]
async fn should_return_401_if_invalid_token() {
    let app = TestApp::new().await;

    let response = app.post_verify_token(&json!({
        "token": "woeifjweoijf",
    })).await;

    assert_eq!(response.status().as_u16(), 401);
}

#[tokio::test]
async fn should_return_401_if_banned_token() {
    let app = TestApp::new().await;

    let email = Email::parse(get_random_email()).expect("email should be parseable");
    let valid_token = generate_auth_token(&email).expect("should generate token");
    {
        let mut banned_store = app.banned_token_store.write().await;
        banned_store.add_token(valid_token.clone());
    }
    let response = app.post_verify_token(&json!({
        "token": valid_token,
    })).await;

    assert_eq!(response.status().as_u16(), 401);
}
