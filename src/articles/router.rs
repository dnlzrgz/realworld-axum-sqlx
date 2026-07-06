use crate::{
    articles::handlers::{
        create_article, delete_article, favorite_article, get_article, get_tags,
        unfavorite_article, update_article,
    },
    state::AppState,
};
use axum::{
    Router,
    routing::{get, post},
};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/articles", post(create_article))
        .route(
            "/articles/{slug}",
            get(get_article).put(update_article).delete(delete_article),
        )
        .route(
            "/articles/{slug}/favorite",
            get(favorite_article).delete(unfavorite_article),
        )
        .route("/tags", get(get_tags))
}
