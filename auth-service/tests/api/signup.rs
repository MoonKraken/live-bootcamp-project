use auth_service::{routes::SignupResponse, ErrorResponse};

use crate::helpers::{get_random_email, TestApp};

#[tokio::test]
async fn should_return_422_if_malformed_input() {
    let mut app = TestApp::new().await;

    // TODO: add more malformed input test cases
    let test_cases = [serde_json::json!({
        "password": "password123",
        "requires2FA": true,
        // "email": random_email,
    })];

    for test_case in test_cases.iter() {
        let response = app.post_signup(test_case).await; // call `post_signup`
        assert_eq!(
            response.status().as_u16(),
            422,
            "Failed for input: {:?}",
            test_case
        );
    }
    app.clean_up().await;
}

#[tokio::test]
async fn should_return_201_if_valid_input() {
    let mut app = TestApp::new().await;
    let random_email = get_random_email(); // Call helper method to generate email
    let test_cases = [serde_json::json!({
        "password": "password123",
        "requires2FA": true,
        "email": random_email,
    })];
    for test_case in test_cases.iter() {
        let response = app.post_signup(test_case).await; // call `post_signup`
        let expected_response = SignupResponse {
            message: "User created successfully!".to_string(),
        };
        assert_eq!(
            response
                .json::<SignupResponse>()
                .await
                .expect("could not deserialize"),
            expected_response,
            "Failed for input: {:?}",
            test_case
        );
    }
    app.clean_up().await;
}

#[tokio::test]
async fn should_return_400_if_invalid_input() {
    // The signup route should return a 400 HTTP status code if an invalid input is sent.
    // The input is considered invalid if:
    // - The email is empty or does not contain '@'
    // - The password is less than 8 characters

    // Create an array of invalid inputs. Then, iterate through the array and
    // make HTTP calls to the signup route. Assert a 400 HTTP status code is returned.
    let mut app = TestApp::new().await;
    let test_cases = [
        serde_json::json!({
            "password": "password123",
            "requires2FA": true,
            "email": "woiefjwoeijf",
        }),
        serde_json::json!({
            "password": "password123",
            "requires2FA": true,
            "email": "",
        }),
    ];
    for test_case in test_cases.iter() {
        let response = app.post_signup(test_case).await; // call `post_signup`
        assert_eq!(
            response.status().as_u16(),
            400,
            "Failed for input: {:?}",
            test_case
        );

        assert_eq!(
            response
                .json::<ErrorResponse>()
                .await
                .expect("Could not deserialize response body to ErrorResponse")
                .error,
            "Invalid credentials".to_owned()
        );
    }
    app.clean_up().await;
}

#[tokio::test]
async fn should_return_409_if_email_already_exists() {
    // Call the signup route twice. The second request should fail with a 409 HTTP status code
    let mut app = TestApp::new().await;
    let random_email = get_random_email(); // Call helper method to generate email
    let test_data = serde_json::json!({
        "password": "password123",
        "requires2FA": true,
        "email": random_email,
    });
    let _ = app.post_signup(&test_data).await; // call `post_signup`
    let response = app.post_signup(&test_data).await; // call `post_signup`
    assert_eq!(response.status().as_u16(), 409);

    assert_eq!(
        response
            .json::<ErrorResponse>()
            .await
            .expect("Could not deserialize response body to ErrorResponse")
            .error,
        "User already exists".to_owned()
    );
    app.clean_up().await;
}
