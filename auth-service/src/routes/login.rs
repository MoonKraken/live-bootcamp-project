use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use axum_extra::extract::CookieJar;
use serde::{Deserialize, Serialize};

use crate::app_state::AppState;
use crate::domain::data_stores::{LoginAttemptId, TwoFACode};
use crate::domain::email::Email;
use crate::domain::error::AuthAPIError;
use crate::domain::password::Password;
use crate::utils::auth::generate_auth_cookie;

#[derive(Deserialize, Debug)]
pub struct LoginRequest {
    email: String,
    password: String,
}

pub async fn login_handler(
    State(state): State<AppState>,
    jar: CookieJar,
    Json(request): Json<LoginRequest>,
) -> (CookieJar, Result<impl IntoResponse, AuthAPIError>) {
    let email = Email::parse(request.email);
    let password = Password::parse(request.password);

    let (email, password) = if let (Ok(email), Ok(password)) = (email, password) {
        (email, password)
    } else {
        return (jar, Err(AuthAPIError::InvalidCredentials));
    };

    let user_store = state.user_store.read().await;

    if let Err(_) = user_store.validate_user(&email, &password) {
        return (jar, Err(AuthAPIError::IncorrectCredentials));
    }

    let user = user_store.get_user(&email);

    let user = if let Ok(user) = user {
        user
    } else {
        return (jar, Err(AuthAPIError::IncorrectCredentials));
    };

    // TODO figure out why I couldn't put each of these branches in a function
    // they give me type errors in lib.rs when i do so
    if user.requires_2fa {
        let login_attempt_id = LoginAttemptId::default();
        let two_fa_code = TwoFACode::default();

        {
            let mut two_fa_store = state.two_fa_code_store.write().await;
            if let Err(_) = two_fa_store
                .add_code(email.clone(), login_attempt_id.clone(), two_fa_code.clone())
            {
                return (jar, Err(AuthAPIError::UnexpectedError));
            }
        }

        let email_client = state.email_client.write().await;
        if let Err(_) = email_client.send_email(&email, "login now", two_fa_code.as_ref()) {
            return (jar, Err(AuthAPIError::UnexpectedError));
        }

        let two_factor_response = TwoFactorAuthResponse {
            message: "2FA required".to_string(),
            login_attempt_id: login_attempt_id.as_ref().to_string(),
        };
        (
            jar,
            Ok((
                StatusCode::PARTIAL_CONTENT,
                axum::Json(LoginResponse::TwoFactorAuth(two_factor_response)),
            )),
        )
    } else {
        let auth_cookie = generate_auth_cookie(&email);

        let auth_cookie = if let Ok(auth_cookie) = auth_cookie {
            auth_cookie
        } else {
            return (jar, Err(AuthAPIError::UnexpectedError));
        };

        let updated_jar = jar.add(auth_cookie);
        let response = axum::Json(LoginResponse::RegularAuth);
        (updated_jar, Ok((StatusCode::OK, response)))
    }
}
// New!
// async fn handle_2fa(
//     jar: CookieJar,
// ) -> (
//     CookieJar,
//     Result<(StatusCode, Json<LoginResponse>), AuthAPIError>,
// ) {
//     // let auth_cookie = generate_auth_cookie(&email);

//     // let auth_cookie = if let Ok(auth_cookie) = auth_cookie {
//     //     auth_cookie
//     // } else {
//     //     return (jar, Err(AuthAPIError::UnexpectedError));
//     // };

//     // let updated_jar = jar.add(auth_cookie);
//     let two_factor_response = TwoFactorAuthResponse {
//         message: "2FA required".to_string(),
//         login_attempt_id: "123456".to_string(),
//     };
//     (
//         jar,
//         Ok((
//             StatusCode::OK,
//             axum::Json(LoginResponse::TwoFactorAuth(two_factor_response)),
//         )),
//     )
// }

// New!
// async fn handle_no_2fa(
//     email: &Email,
//     jar: CookieJar,
// ) -> (
//     CookieJar,
//     Result<(StatusCode, Json<LoginResponse>), AuthAPIError>,
// ) {
//     let auth_cookie = generate_auth_cookie(&email);

//     let auth_cookie = if let Ok(auth_cookie) = auth_cookie {
//         auth_cookie
//     } else {
//         return (jar, Err(AuthAPIError::UnexpectedError));
//     };

//     let updated_jar = jar.add(auth_cookie);
//     let response = axum::Json(LoginResponse::RegularAuth);
//     (updated_jar, Ok((StatusCode::OK, response)))
// }

// The login route can return 2 possible success responses.
// This enum models each response!
#[derive(Debug, Deserialize, Serialize)]
#[serde(untagged)]
pub enum LoginResponse {
    RegularAuth,
    TwoFactorAuth(TwoFactorAuthResponse),
}

// If a user requires 2FA, this JSON body should be returned!
#[derive(Debug, Serialize, Deserialize)]
pub struct TwoFactorAuthResponse {
    pub message: String,
    #[serde(rename = "loginAttemptId")]
    pub login_attempt_id: String,
}
