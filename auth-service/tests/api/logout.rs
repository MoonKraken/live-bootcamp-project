use auth_service::{
    domain::email::Email,
    utils::{auth::generate_auth_cookie, constants::JWT_COOKIE_NAME},
};
use reqwest::Url;
use secrecy::Secret;

use crate::helpers::{get_random_email, TestApp};

#[tokio::test]
async fn should_return_400_if_jwt_cookie_missing() {
    let mut app = TestApp::new().await;

    let response = app.post_logout().await;

    assert_eq!(response.status().as_u16(), 400);
    app.clean_up().await;
}

#[tokio::test]
async fn should_return_401_if_invalid_token() {
    let mut app = TestApp::new().await;

    // add invalid cookie
    app.cookie_jar.add_cookie_str(
        &format!(
            "{}=invalid; HttpOnly; SameSite=Lax; Secure; Path=/",
            JWT_COOKIE_NAME
        ),
        &Url::parse("http://127.0.0.1").expect("Failed to parse URL"),
    );

    let response = app.post_logout().await;

    assert_eq!(response.status().as_u16(), 401);
    app.clean_up().await;
}

#[tokio::test]
async fn should_return_200_if_valid_jwt_cookie() {
    let mut app = TestApp::new().await;

    let email = Email::parse(Secret::new(get_random_email())).expect("email should be parseable");
    let cookie = generate_auth_cookie(&email).expect("should generate auth cookie");
    app.cookie_jar.add_cookie_str(
        &cookie.to_string(),
        &Url::parse("http://127.0.0.1").expect("Failed to parse URL"),
    );

    let response = app.post_logout().await;
    assert_eq!(response.status().as_u16(), 200);

    {
        let banned_store = app.banned_token_store.read().await;
        let contains_token = banned_store
            .contains_token(&Secret::new(cookie.value().to_string()))
            .await;

        match contains_token {
            Ok(true) => { /*happy path*/ }
            _ => panic!("Did not contain token"),
        }
    }
    app.clean_up().await;
}

#[tokio::test]
async fn should_return_400_if_logout_called_twice_in_a_row() {
    let mut app = TestApp::new().await;

    let email = Email::parse(Secret::new(get_random_email())).expect("email should be parseable");
    let cookie = generate_auth_cookie(&email).expect("should generate auth cookie");
    app.cookie_jar.add_cookie_str(
        &cookie.to_string(),
        &Url::parse("http://127.0.0.1").expect("Failed to parse URL"),
    );

    let _ = app.post_logout().await;
    let response = app.post_logout().await;
    assert_eq!(response.status().as_u16(), 400);
    app.clean_up().await;
}
