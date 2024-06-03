use crate::helpers::{get_random_email, TestApp};
use auth_service::{domain::email::Email, routes::TwoFactorAuthResponse, utils::constants::JWT_COOKIE_NAME};
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

#[tokio::test]
async fn should_return_200_if_correct_credentials() {
    // Call the log-in route with incorrect credentials and assert
    // that a 401 HTTP status code is returned along with the appropriate error message.
    let app = TestApp::new().await;
    let random_email = get_random_email(); // Call helper method to generate email
    let test_data = serde_json::json!({
        "password": "password123",
        "requires2FA": false,
        "email": random_email,
    });

    // first add the user
    let _ = app.post_signup(&test_data).await; // call `post_signup`
    let body = json!({
        "email": random_email,
        "password": "password123",
    });

    // the try to authenticate them
    let response = app.post_login(&body).await;

    assert_eq!(response.status().as_u16(), 200);

    let auth_cookie = response
        .cookies()
        .find(|cookie| cookie.name() == JWT_COOKIE_NAME)
        .expect("No auth cookie found");

    assert!(!auth_cookie.value().is_empty());
}

#[tokio::test]
async fn should_return_206_if_valid_credentials_and_2fa_enabled() {
    let app = TestApp::new().await;
    let random_email = get_random_email(); // Call helper method to generate email
    let test_data = serde_json::json!({
        "password": "password123",
        "requires2FA": true,
        "email": random_email,
    });

    // first add the user
    let _ = app.post_signup(&test_data).await; // call `post_signup`
    let body = json!({
        "email": random_email,
        "password": "password123",
    });
    let response = app.post_login(&body).await;
    assert_eq!(response.status().as_u16(), 206);
    let json_body = response
        .json::<TwoFactorAuthResponse>()
        .await
        .expect("Could not deserialize response body to TwoFactorAuthResponse");

    assert_eq!(json_body.message, "2FA required".to_owned());

    let id = json_body.login_attempt_id;
    let two_fa_stuff = app.two_fa_store.read().expect("should get read lock");
    match two_fa_stuff.get_code(&Email::parse(random_email).expect("parse email")) {
        Ok((_, _)) => {},
        Err(_) => {
            panic!("Email wasn't present in 2FA code store")
        }
    };
}
