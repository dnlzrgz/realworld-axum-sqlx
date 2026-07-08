use axum::{
    Router,
    routing::{delete, get},
};

use crate::{
    comments::handlers::{add_comment, delete_comment, get_article_comments},
    state::AppState,
};

pub fn router() -> Router<AppState> {
    Router::new()
        .route(
            "/articles/{slug}/comments",
            get(get_article_comments).post(add_comment),
        )
        .route(
            "/articles/{slug}/comments/{comment_id}",
            delete(delete_comment),
        )
}
