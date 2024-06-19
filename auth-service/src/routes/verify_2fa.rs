use axum::extract::State;
use axum::http::StatusCode;
use axum::{response::IntoResponse, Json};
use axum_extra::extract::CookieJar;
use serde::Deserialize;

use crate::app_state::AppState;
use crate::domain::data_stores::{LoginAttemptId, TwoFACode};
use crate::domain::error::AuthAPIError;
use crate::domain::Email;
use crate::utils::auth::generate_auth_cookie;
use crate::LoginResponse;

#[derive(Deserialize, Debug)]
pub struct Verify2FARequest {
    email: String,
    #[serde(rename = "loginAttemptId")]
    login_attempt_id: String,
    #[serde(rename = "2FACode")]
    code: String,
}

#[tracing::instrument(name = "Verify 2FA", skip_all)]
pub async fn verify_2fa_handler(
    State(state): State<AppState>,
    jar: CookieJar,
    Json(request): Json<Verify2FARequest>,
) -> (CookieJar, Result<impl IntoResponse, AuthAPIError>) {
    // Update this to a custom message!
    let email = if let Ok(val) = Email::parse(request.email) {
        val
    } else {
        return (jar, Err(AuthAPIError::InvalidCredentials));
    };

    let code = if let Ok(val) = TwoFACode::parse(request.code) {
        val
    } else {
        return (jar, Err(AuthAPIError::InvalidCredentials));
    };

    let id = if let Ok(val) = LoginAttemptId::parse(request.login_attempt_id) {
        val
    } else {
        return (jar, Err(AuthAPIError::InvalidCredentials));
    };

    let mut two_fa_code_store = state.two_fa_code_store.write().await;

    let entry = if let Ok(val) = two_fa_code_store.get_code(&email).await {
        val
    } else {
        return (jar, Err(AuthAPIError::IncorrectCredentials));
    };

    if entry == (id, code) {
        let _ = if let Ok(val) = two_fa_code_store.remove_code(&email).await {
            val
        } else {
            return (jar, Err(AuthAPIError::InvalidCredentials));
        };

        let auth_cookie = generate_auth_cookie(&email);

        let auth_cookie = match auth_cookie {
            Ok(auth_cookie) => auth_cookie,
            Err(e) => return (jar, Err(AuthAPIError::UnexpectedError(e))),
        };

        let updated_jar = jar.add(auth_cookie);
        let response = axum::Json(LoginResponse::RegularAuth);
        (updated_jar, Ok((StatusCode::OK, response)))
    } else {
        (jar, Err(AuthAPIError::IncorrectCredentials))
    }
}
