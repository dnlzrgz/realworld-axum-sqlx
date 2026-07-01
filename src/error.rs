use axum::Json;
use axum::http::{HeaderMap, HeaderValue, StatusCode, header::WWW_AUTHENTICATE};
use axum::response::IntoResponse;
use serde_json::json;
use std::borrow::Cow;
use std::collections::HashMap;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Return `401 Unauthorized`
    #[error("authentication required")]
    Unauthorized,

    /// Return `403 Forbidden`
    #[error("current user is not allowed to perform that action")]
    Forbidden,

    /// Return `404 Not Found`
    #[error("request path not found")]
    NotFound,

    /// Return `422 Unprocessable Entity`
    #[error("error while processing the request body")]
    UnprocessableEntity {
        errors: HashMap<Cow<'static, str>, Vec<Cow<'static, str>>>,
    },

    /// Automatically return `500 Internal Server Error` on a `sqlx::Error`.
    #[error("an error occurred with the database")]
    Sqlx(#[from] sqlx::Error),

    /// Return `500 Internal Server Error` on a `anyhow::Error`.
    #[error("an internal error occurred")]
    Anyhow(#[from] anyhow::Error),
}

impl IntoResponse for Error {
    fn into_response(self) -> axum::response::Response {
        let (status, message) = match &self {
            Self::Unauthorized => {
                // Include the `WWW-Authenticate` challenge required in the specification
                // for the `401 Unauthorized` response code:
                let mut headers = HeaderMap::new();
                headers.insert(WWW_AUTHENTICATE, HeaderValue::from_static("Token"));
                return (StatusCode::UNAUTHORIZED, headers, self.to_string()).into_response();
            }
            Self::Forbidden => (StatusCode::FORBIDDEN, self.to_string()),
            Self::NotFound => (StatusCode::NOT_FOUND, self.to_string()),
            Self::UnprocessableEntity { errors } => {
                return (
                    StatusCode::UNPROCESSABLE_ENTITY,
                    Json(json!({"errors": errors})),
                )
                    .into_response();
            }
            Self::Sqlx(e) => {
                tracing::error!("sqlx error: {:?}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "internal server error".into(),
                )
            }
            Self::Anyhow(e) => {
                tracing::error!("internal error: {:#}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "internal server error".into(),
                )
            }
        };

        (status, Json(json!({ "errors": { "body": [message] } }))).into_response()
    }
}

impl Error {
    pub fn validation(
        field: impl Into<Cow<'static, str>>,
        message: impl Into<Cow<'static, str>>,
    ) -> Self {
        let mut errors = HashMap::new();

        errors.insert(field.into(), vec![message.into()]);

        Self::UnprocessableEntity { errors }
    }
    pub fn from_sqlx(e: sqlx::Error) -> Self {
        match e {
            sqlx::Error::Database(err) => match err.constraint() {
                Some("users_email_key") => Self::validation("email", "has already been taken"),
                Some("users_username_key") => {
                    Self::validation("username", "has already been taken")
                }
                Some("user_cannot_follow_self") => Self::Forbidden,
                _ => Self::Sqlx(sqlx::Error::Database(err)),
            },
            e => Self::Sqlx(e),
        }
    }
}
