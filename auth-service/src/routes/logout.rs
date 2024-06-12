use axum::{extract::State, response::IntoResponse};
use axum_extra::extract::{cookie::Cookie, CookieJar};
use axum::http::StatusCode;

use crate::{app_state::AppState, domain::error::AuthAPIError, utils::{auth::validate_token, constants::JWT_COOKIE_NAME}};
pub async fn logout_handler(
    State(state): State<AppState>,
    jar: CookieJar,
) -> (CookieJar, Result<impl IntoResponse, AuthAPIError>) {
    // Retrieve JWT cookie from the `CookieJar`
    // Return AuthAPIError::MissingToken is the cookie is not found
    let token = match jar.get(JWT_COOKIE_NAME) {
        Some(cookie) => cookie.value().to_owned(),
        None => return (jar, Err(AuthAPIError::MissingToken)),
    };

    // TODO: Validate JWT token by calling `validate_token` from the auth service.
    // If the token is valid you can ignore the returned claims for now.
    // Return AuthAPIError::InvalidToken is validation fails.

    let validation = validate_token(&token, &state.banned_token_store).await;
    match validation {
        Ok(_) => {},
        Err(_) => return (jar, Err(AuthAPIError::InvalidToken))
    }

    //remove the cookie
    let jar = jar.remove(Cookie::from(JWT_COOKIE_NAME));

    //add to the banned list
    let mut banned_token_store = state.banned_token_store.write().await;
    match banned_token_store.add_token(token).await {
        Ok(()) => (jar, Ok(StatusCode::OK)),
        _ => (jar, Err(AuthAPIError::UnexpectedError))
    }

}
