use axum::{
    Json,
    extract::{Path, State},
};
use futures::TryStreamExt;
use uuid::Uuid;

use crate::{
    comments::dto::{AddComment, CommentBody, CommentFromQuery, MultipleCommentsBody},
    error::Error,
    extractor::{AuthUser, MaybeAuthUser},
    state::AppState,
};

pub async fn get_article_comments(
    maybe_auth_user: MaybeAuthUser,
    State(state): State<AppState>,
    Path(slug): Path<String>,
) -> Result<Json<MultipleCommentsBody>, Error> {
    // With this, we can return 404 if the article slug was not found.
    let article_id = sqlx::query_scalar!("select article_id from articles where slug = $1", slug)
        .fetch_optional(&state.db)
        .await?
        .ok_or(Error::NotFound)?;

    let comments = sqlx::query_as!(
        CommentFromQuery,
        r#"
            select
                comment_id,
                comment.created_at,
                comment.updated_at,
                comment.body,
                author.username author_username,
                author.bio author_bio,
                author.image author_image,
                exists(select 1 from follow where following_user_id = author.user_id and follower_user_id = $1) "following_author!"
            from comments comment
            inner join users author using (user_id)
            where article_id = $2
            order by created_at
        "#,
        maybe_auth_user.user_id(),
        article_id
    )
        .fetch(&state.db)
        .map_ok(CommentFromQuery::into_comment)
        .try_collect()
        .await?;

    Ok(Json(MultipleCommentsBody { comments }))
}

pub async fn add_comment(
    auth_user: AuthUser,
    State(state): State<AppState>,
    Path(slug): Path<String>,
    req: Json<CommentBody<AddComment>>,
) -> Result<Json<CommentBody>, Error> {
    let comment = sqlx::query_as!(
        CommentFromQuery,
        r#"
            with inserted_comment as (
                insert into comments(article_id, user_id, body)
                select article_id, $1, $2
                from articles
                where slug = $3
                returning comment_id, created_at, updated_at, body
            )
            select
                comment_id,
                comment.created_at,
                comment.updated_at,
                body,
                author.username author_username,
                author.bio author_bio,
                author.image author_image,
                false "following_author!"
            from inserted_comment comment
            inner join users author on user_id = $1
        "#,
        auth_user.user_id,
        req.comment.body,
        slug
    )
    .fetch_optional(&state.db)
    .await?
    .ok_or(Error::NotFound)?
    .into_comment();

    Ok(Json(CommentBody { comment }))
}

pub async fn delete_comment(
    auth_user: AuthUser,
    State(state): State<AppState>,
    Path((slug, comment_id)): Path<(String, Uuid)>,
) -> Result<(), Error> {
    let result = sqlx::query!(
        r#"
            with deleted_comment as (
                delete from comments
                where
                    comment_id = $1
                    and article_id in (select article_id from articles where slug = $2)
                    and user_id = $3
                returning 1
            )
            select
                exists(
                    select 1 from comments
                    inner join articles using (article_id)
                    where comment_id = $1 and slug = $2
                ) "existed!",
                exists(select 1 from deleted_comment) "deleted!"
        "#,
        comment_id,
        slug,
        auth_user.user_id
    )
    .fetch_one(&state.db)
    .await?;

    if result.deleted {
        Ok(())
    } else if result.existed {
        Err(Error::Forbidden)
    } else {
        Err(Error::NotFound)
    }
}
