use crate::helpers::{get_random_email, TestApp};
use serde_json::json;

#[tokio::test]
async fn should_return_422_if_malformed_credentials() {
    let app = TestApp::new().await;

    let body = json!(
        "
weoifjwofiej
"
    );

    let response = app.post_login(&body).await;

    assert_eq!(response.status().as_u16(), 422);
}

#[tokio::test]
async fn should_return_400_if_invalid_input() {
    // Call the log-in route with invalid credentials and assert that a
    // 400 HTTP status code is returned along with the appropriate error message.
    let app = TestApp::new().await;

    let body = json!({
        "email": "woeifjwioejfoj",
        "password": ""
    }
    );

    let response = app.post_login(&body).await;

    assert_eq!(response.status().as_u16(), 400);
}

#[tokio::test]
async fn should_return_401_if_incorrect_credentials() {
    // Call the log-in route with incorrect credentials and assert
    // that a 401 HTTP status code is returned along with the appropriate error message.
    let app = TestApp::new().await;
    let body = json!({
        "email": "ken@cttm.io",
        "password": "owiejfwoiejfoij",
    });

    let response = app.post_login(&body).await;

    assert_eq!(response.status().as_u16(), 401);
}
