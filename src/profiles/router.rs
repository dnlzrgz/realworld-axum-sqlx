use crate::{
    profiles::handlers::{follow_user, get_user_profile, unfollow_user},
    state::AppState,
};
use axum::{
    Router,
    routing::{get, post},
};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/profiles/{username}", get(get_user_profile))
        .route(
            "/profiles/{username}/follow",
            post(follow_user).delete(unfollow_user),
        )
}
