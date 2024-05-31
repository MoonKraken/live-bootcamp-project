use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use axum_extra::extract::CookieJar;
use serde::Deserialize;

use crate::app_state::AppState;
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
        return (jar, Err(AuthAPIError::InvalidCredentials))
    };

    let user_store = state.user_store.read().unwrap();

    if let Err(_) = user_store.validate_user(&email, &password) {
        return (jar, Err(AuthAPIError::IncorrectCredentials));
    }

    let user = user_store
        .get_user(&email);

    let _user = if let Ok(user) = user {
        user
    } else {
        return (jar, Err(AuthAPIError::IncorrectCredentials));
    };

    let auth_cookie = generate_auth_cookie(&email);

    let auth_cookie = if let Ok(auth_cookie) = auth_cookie {
        auth_cookie
    } else {
        return (jar, Err(AuthAPIError::UnexpectedError))
    };

    let updated_jar = jar.add(auth_cookie);
    (updated_jar, Ok(StatusCode::OK.into_response()))
}
