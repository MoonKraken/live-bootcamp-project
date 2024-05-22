use axum::{extract::State, response::IntoResponse};
use axum::http::StatusCode;
use axum::Json;
use serde::{Deserialize, Serialize};

use crate::app_state::AppState;
use crate::domain::error::AuthAPIError;
use crate::domain::user::User;
use crate::services::hashmap_user_store::UserStoreError;

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
        return Err(AuthAPIError::InvalidCredentials)
    }
    // Create a new `User` instance using data in the `request`
    let mut user_store = state.user_store.write().unwrap();

    let add_res = user_store.add_user(User {
        email: request.email,
        password: request.password,
        requires_2fa: request.requires_2fa,
    });

    if let Err(e) = add_res {
        if e == UserStoreError::UserAlreadyExists {
            return Err(AuthAPIError::UserAlreadyExists)
        }
    }

    let response = Json(SignupResponse {
        message: "User created successfully!".to_string(),
    });

    Ok((StatusCode::CREATED, response))
}
