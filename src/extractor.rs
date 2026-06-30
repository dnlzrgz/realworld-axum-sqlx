use axum::extract::{FromRef, FromRequestParts, State};
use axum::http::header::AUTHORIZATION;
use axum::http::request::Parts;
use chrono::{Duration, Utc};
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};
use serde::{Deserialize, Serialize};

use uuid::Uuid;

use crate::error::Error;
use crate::state::AppState;

const SESSION_DAYS: i64 = 7;
const SCHEME_PREFIX: &str = "Token ";

/// Add this as a parameter to a handler function to require the user to be logged.
pub struct AuthUser {
    pub user_id: Uuid,
}

/// Add this as a parameter to a handler function to optionally check if the user
/// is logged in.
pub struct MaybeAuthUser(pub Option<AuthUser>);

#[derive(Serialize, Deserialize)]
struct AuthUserClaims {
    user_id: Uuid,
    /// Standard JWT `exp` claim, seconds since epoch.
    exp: i64,
}

impl AuthUser {
    pub fn to_jwt(&self, state: &AppState) -> String {
        let exp = (Utc::now() + Duration::days(SESSION_DAYS)).timestamp();

        let claims = AuthUserClaims {
            user_id: self.user_id,
            exp,
        };

        encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(state.jwt_secret.as_bytes()),
        )
        .expect("JWT encoding should be infallible")
    }

    fn from_authorization(state: &AppState, auth_header: &str) -> Result<Self, Error> {
        let token = auth_header.strip_prefix(SCHEME_PREFIX).ok_or_else(|| {
            tracing::debug!(
                "Authorization header is using the wrong scheme: {:?}",
                auth_header
            );
            Error::Unauthorized
        })?;

        let data = decode::<AuthUserClaims>(
            token,
            &DecodingKey::from_secret(state.jwt_secret.as_bytes()),
            &Validation::default(),
        )
        .map_err(|e| {
            tracing::debug!("JWT failed to verify: {e}");
            Error::Unauthorized
        })?;

        // `jsonwebtoken`'s default `Validation` already checks `exp` for us.
        Ok(Self {
            user_id: data.claims.user_id,
        })
    }
}

impl MaybeAuthUser {
    pub fn user_id(&self) -> Option<Uuid> {
        self.0.as_ref().map(|auth_user| auth_user.user_id)
    }
}

impl<S> FromRequestParts<S> for AuthUser
where
    AppState: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = Error;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let State(app_state) = State::<AppState>::from_request_parts(parts, state)
            .await
            .expect("AppState was not added");

        let auth_header = parts
            .headers
            .get(AUTHORIZATION)
            .ok_or(Error::Unauthorized)?
            .to_str()
            .map_err(|_| {
                tracing::debug!("Authorization header is not UTF-8");
                Error::Unauthorized
            })?;

        Self::from_authorization(&app_state, auth_header)
    }
}

impl<S> FromRequestParts<S> for MaybeAuthUser
where
    AppState: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = Error;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let State(app_state) = State::<AppState>::from_request_parts(parts, state)
            .await
            .expect("AppState was not added");

        let maybe_user = match parts.headers.get(AUTHORIZATION) {
            Some(header) => {
                let header = header.to_str().map_err(|_| {
                    tracing::debug!("Authorization header is not UTF-8");
                    Error::Unauthorized
                })?;
                Some(AuthUser::from_authorization(&app_state, header)?)
            }
            None => None,
        };

        Ok(Self(maybe_user))
    }
}
