use axum::response::IntoResponse;
use axum_extra::extract::{cookie::Cookie, CookieJar};
use axum::http::StatusCode;

use crate::{domain::error::AuthAPIError, utils::{auth::validate_token, constants::JWT_COOKIE_NAME}};
pub async fn logout_handler(
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

    let validation = validate_token(&token).await;
    match validation {
        Ok(_) => {},
        Err(_) => return (jar, Err(AuthAPIError::InvalidToken))
    }

    //remove the cookie
    let jar = jar.remove(Cookie::from(JWT_COOKIE_NAME));

    (jar, Ok(StatusCode::OK))
}
