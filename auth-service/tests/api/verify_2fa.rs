use auth_service::{
    domain::{
        data_stores::{LoginAttemptId, TwoFACode},
        Email,
    }, routes::LoginResponse, utils::constants::JWT_COOKIE_NAME
};
use serde_json::json;

use crate::helpers::{get_random_email, TestApp};

#[tokio::test]
async fn should_return_422_if_malformed_input() {
    // Call the log-in route with incorrect credentials and assert
    // that a 401 HTTP status code is returned along with the appropriate error message.
    let app = TestApp::new().await;

    let response = app.post_verify_2fa(&json!({})).await;
    assert_eq!(response.status().as_u16(), 422);
}

#[tokio::test]
async fn should_return_400_if_invalid_input() {
    let app = TestApp::new().await;

    let invalid_inputs = vec![
        json!({
            "email": get_random_email(),
            "loginAttemptId": "oij",
            "2FACode": TwoFACode::default(),
        }),
        json!({
            "email": get_random_email(),
            "loginAttemptId": LoginAttemptId::default(),
            "2FACode": "12",
        }),
        json!({
            "email": "",
            "loginAttemptId": LoginAttemptId::default(),
            "2FACode": TwoFACode::default(),
        }),
    ];

    for input in invalid_inputs {
        let res = app.post_verify_2fa(&input).await;
        assert_eq!(res.status().as_u16(), 400);
    }
}

#[tokio::test]
async fn should_return_401_if_incorrect_credentials() {
    let app = TestApp::new().await;

    let random_email = get_random_email(); // Call helper method to generate email
    let user_to_create = serde_json::json!({
        "password": "password123",
        "requires2FA": true,
        "email": random_email,
    });

    // first add the user
    let _ = app.post_signup(&user_to_create).await; // call `post_signup`
    let login_res = app
        .post_login(&json!({
            "email": random_email,
            "password": "password123",
        }))
        .await;

    let login_response_json = login_res
        .json::<LoginResponse>()
        .await
        .expect("Could not deserialize response body to LoginResponse");

    match login_response_json {
        LoginResponse::TwoFactorAuth(two_fa_respons) => {
            // let email = Email::parse(random_email.clone()).unwrap();
            // let code = app.two_fa_store.read().unwrap().get_code(&email).unwrap();
            let verify_res = app
                .post_verify_2fa(&json!({
                    "email": random_email,
                    "loginAttemptId": two_fa_respons.login_attempt_id,
                    "2FACode": "111111", //(hopefully) incorrect code!
                }))
                .await;

            assert_eq!(verify_res.status().as_u16(), 401);
        }
        _ => panic!("two factor auth response expected, did not get one"),
    }
}

#[tokio::test]
async fn should_return_401_if_old_code() {
    // Call login twice. Then, attempt to call verify-fa with the 2FA code from the first login requet. This should fail.
    let app = TestApp::new().await;

    let random_email = get_random_email(); // Call helper method to generate email
    let user_to_create = serde_json::json!({
        "password": "password123",
        "requires2FA": true,
        "email": random_email,
    });

    let login_request = json!({
        "email": random_email,
        "password": "password123",
    });
    // first add the user
    let _ = app.post_signup(&user_to_create).await; // call `post_signup`
                                                    // first login, but we don't care about the response
    let _ = app.post_login(&login_request).await;

    let email = Email::parse(random_email.clone()).unwrap();
    let first_code = app.two_fa_store.read().await.get_code(&email).await.unwrap().1;

    // second login attempt, the code we just grabbed shoudl be invalidated
    let login_res = app.post_login(&login_request).await;
    let login_response_json = login_res
        .json::<LoginResponse>()
        .await
        .expect("Could not deserialize response body to LoginResponse");

    match login_response_json {
        LoginResponse::TwoFactorAuth(two_fa_respons) => {
            // let code = app.two_fa_store.read().unwrap().get_code(&email).unwrap();
            let verify_res = app
                .post_verify_2fa(&json!({
                    "email": random_email,
                    "loginAttemptId": two_fa_respons.login_attempt_id,
                    "2FACode": first_code, //(hopefully) incorrect code!
                }))
                .await;

            assert_eq!(verify_res.status().as_u16(), 401);
        }
        _ => panic!("two factor auth response expected, did not get one"),
    }
}

#[tokio::test]
async fn should_return_200_if_correct_code() {
    // Make sure to assert the auth cookie gets set
    let app = TestApp::new().await;

    let random_email = get_random_email(); // Call helper method to generate email
    let user_to_create = serde_json::json!({
        "password": "password123",
        "requires2FA": true,
        "email": random_email,
    });

    let login_request = json!({
        "email": random_email,
        "password": "password123",
    });
    // first add the user
    let _ = app.post_signup(&user_to_create).await; // call `post_signup`
                                                    // first login, but we don't care about the response

    let email = Email::parse(random_email.clone()).unwrap();

    // second login attempt, the code we just grabbed shoudl be invalidated
    let login_res = app.post_login(&login_request).await;
    let login_response_json = login_res
        .json::<LoginResponse>()
        .await
        .expect("Could not deserialize response body to LoginResponse");

    let code = app.two_fa_store.read().await.get_code(&email).await.unwrap().1;
    match login_response_json {
        LoginResponse::TwoFactorAuth(two_fa_respons) => {
            // let code = app.two_fa_store.read().unwrap().get_code(&email).unwrap();
            let verify_res = app
                .post_verify_2fa(&json!({
                    "email": random_email,
                    "loginAttemptId": two_fa_respons.login_attempt_id,
                    "2FACode": code, //(hopefully) incorrect code!
                }))
                .await;

            assert_eq!(verify_res.status().as_u16(), 200);

            //make sure cookie is set properly
            let auth_cookie = verify_res
                .cookies()
                .find(|cookie| cookie.name() == JWT_COOKIE_NAME)
                .expect("No auth cookie found");

            assert!(!auth_cookie.value().is_empty());
        }
        _ => panic!("two factor auth response expected, did not get one"),
    }
}

#[tokio::test]
async fn should_return_401_if_same_code_twice() {
    // Make sure to assert the auth cookie gets set
    let app = TestApp::new().await;

    let random_email = get_random_email(); // Call helper method to generate email
    let user_to_create = serde_json::json!({
        "password": "password123",
        "requires2FA": true,
        "email": random_email,
    });

    let login_request = json!({
        "email": random_email,
        "password": "password123",
    });
    // first add the user
    let _ = app.post_signup(&user_to_create).await; // call `post_signup`
                                                    // first login, but we don't care about the response

    let email = Email::parse(random_email.clone()).unwrap();

    // second login attempt, the code we just grabbed shoudl be invalidated
    let login_res = app.post_login(&login_request).await;
    let login_response_json = login_res
        .json::<LoginResponse>()
        .await
        .expect("Could not deserialize response body to LoginResponse");

    let code = app.two_fa_store.read().await.get_code(&email).await.unwrap().1;
    match login_response_json {
        LoginResponse::TwoFactorAuth(two_fa_respons) => {
            // let code = app.two_fa_store.read().unwrap().get_code(&email).unwrap();
            let verify_res = app
                .post_verify_2fa(&json!({
                    "email": random_email,
                    "loginAttemptId": two_fa_respons.login_attempt_id,
                    "2FACode": code, //(hopefully) incorrect code!
                }))
                .await;

            assert_eq!(verify_res.status().as_u16(), 200);

            //make sure cookie is set properly
            let auth_cookie = verify_res
                .cookies()
                .find(|cookie| cookie.name() == JWT_COOKIE_NAME)
                .expect("No auth cookie found");

            assert!(!auth_cookie.value().is_empty());

            let verify_res_2 = app
                .post_verify_2fa(&json!({
                    "email": random_email,
                    "loginAttemptId": two_fa_respons.login_attempt_id,
                    "2FACode": code, //(hopefully) incorrect code!
                }))
                .await;

            assert_eq!(verify_res_2.status().as_u16(), 401);
        }
        _ => panic!("two factor auth response expected, did not get one"),
    }
}
