use axum::http::StatusCode;
use axum::Json;
use axum::{extract::State, response::IntoResponse};
use serde::{Deserialize, Serialize};

use crate::app_state::AppState;
use crate::domain::data_stores::UserStoreError;
use crate::domain::email::Email;
use crate::domain::error::AuthAPIError;
use crate::domain::password::Password;
use crate::domain::user::User;

#[derive(Deserialize, Debug)]
pub struct SignupRequest {
    pub email: String,
    pub password: String,
    #[serde(rename = "requires2FA")]
    pub requires_2fa: bool,
}

#[derive(Eq, PartialEq, Debug, Serialize, Deserialize)]
pub struct SignupResponse {
    pub message: String,
}

pub async fn signup_handler(
    State(state): State<AppState>,
    Json(request): Json<SignupRequest>,
) -> Result<impl IntoResponse, AuthAPIError> {
    if request.email.is_empty() || !request.email.contains("@") {
        return Err(AuthAPIError::InvalidCredentials);
    }
    // Create a new `User` instance using data in the `request`
    let mut user_store = state.user_store.write().unwrap();

    let email = Email::parse(request.email).map_err(|_| AuthAPIError::InvalidCredentials)?;

    let password =
        Password::parse(request.password).map_err(|_| AuthAPIError::InvalidCredentials)?;

    let add_res = user_store.add_user(User {
        email,
        password,
        requires_2fa: request.requires_2fa,
    });

    if let Err(e) = add_res {
        if e == UserStoreError::UserAlreadyExists {
            return Err(AuthAPIError::UserAlreadyExists);
        }
    }

    let response = Json(SignupResponse {
        message: "User created successfully!".to_string(),
    });

    Ok((StatusCode::CREATED, response))
}
