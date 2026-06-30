use crate::{
    state::AppState,
    users::handlers::{self, update_user},
};
use axum::{
    Router,
    routing::{get, post},
};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/users", post(handlers::create_user))
        .route("/users/login", post(handlers::login_user))
        .route("/user", get(handlers::get_current_user).put(update_user))
}
