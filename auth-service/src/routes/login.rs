use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use serde::Deserialize;

use crate::app_state::{AppState, UserStoreType};
use crate::domain::data_stores::UserStore;
use crate::domain::email::Email;
use crate::domain::error::AuthAPIError;
use crate::domain::password::Password;

#[derive(Deserialize, Debug)]
pub struct LoginRequest {
    email: String,
    password: String,
}
pub async fn login_handler(
    State(state): State<AppState>,
    Json(request): Json<LoginRequest>,
) -> Result<impl IntoResponse, AuthAPIError> {

    let email = Email::parse(request.email).map_err(|_| AuthAPIError::InvalidCredentials)?;

    let password =
        Password::parse(request.password).map_err(|_| AuthAPIError::InvalidCredentials)?;

    let user_store = state.user_store.write().unwrap();

    if let Err(_) = user_store.validate_user(&email, &password) {
        return Err(AuthAPIError::IncorrectCredentials);
    }

    let user = user_store.get_user(&email).map_err(|_| AuthAPIError::IncorrectCredentials)?;

    Ok(StatusCode::OK.into_response())
}
