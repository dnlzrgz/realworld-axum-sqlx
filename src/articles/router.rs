use crate::{
    articles::handlers::{
        create_article, delete_article, favorite_article, feed_articles, get_article, get_tags,
        list_articles, unfavorite_article, update_article,
    },
    state::AppState,
};
use axum::{
    Router,
    routing::{get, post},
};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/articles", post(create_article).get(list_articles))
        .route("/articles/feed", get(feed_articles))
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
