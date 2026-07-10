use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
};
use futures::TryStreamExt;

use crate::{
    articles::{dto::*, utils::slugify},
    error::Error,
    extractor::{AuthUser, MaybeAuthUser},
    state::AppState,
};

pub async fn create_article(
    auth_user: AuthUser,
    State(state): State<AppState>,
    Json(mut req): Json<ArticleBody<CreateArticle>>,
) -> Result<Json<ArticleBody>, Error> {
    let slug = slugify(&req.article.title);
    req.article.tag_list.sort();

    let article = sqlx::query_as!(
        ArticleFromQuery,
        r#"
            with inserted_article as (
                insert into articles (user_id, slug, title, description, body, tag_list)
                values ($1, $2, $3, $4, $5, $6)
                returning 
                    slug, 
                    title, 
                    description, 
                    body, 
                    tag_list, 
                    created_at,
                    updated_at
            )
            select 
                inserted_article.*,
                false "favorited!",
                0::int8 "favorites_count!",
                username author_username,
                bio author_bio,
                image author_image,
                false "following_author!"
            from inserted_article
            inner join users on user_id = $1
        "#,
        auth_user.user_id,
        slug,
        req.article.title,
        req.article.description,
        req.article.body,
        &req.article.tag_list[..],
    )
    .fetch_one(&state.db)
    .await
    .map_err(Error::from_sqlx)?;

    Ok(Json(ArticleBody {
        article: article.into_article(),
    }))
}

pub async fn update_article(
    auth_user: AuthUser,
    State(state): State<AppState>,
    Path(slug): Path<String>,
    Json(req): Json<ArticleBody<UpdateArticle>>,
) -> Result<Json<ArticleBody>, Error> {
    let mut tx = state.db.begin().await.map_err(Error::from_sqlx)?;

    let new_slug = req.article.title.as_deref().map(slugify);

    let article_meta = sqlx::query!(
        "select article_id, user_id from articles where slug = $1 for update",
        slug
    )
    .fetch_optional(&mut *tx)
    .await?
    .ok_or(Error::NotFound)?;

    if article_meta.user_id != auth_user.user_id {
        return Err(Error::Forbidden);
    }

    let article = sqlx::query_as!(
        ArticleFromQuery,
        r#"
            with updated_article as (
                update articles
                set
                    slug = coalesce($1, slug),
                    title = coalesce($2, title),
                    description = coalesce($3, description),
                    body = coalesce($4, body),
                    tag_list = coalesce($5, tag_list)
                where article_id = $6
                returning
                    slug,
                    title,
                    description,
                    body,
                    tag_list,
                    created_at,
                    updated_at
            )
            select
                updated_article.*,
                exists(select 1 from favorites where user_id = $7) "favorited!",
                coalesce(
                    (select count(*) from favorites fav where fav.article_id = $6),
                    0
                ) "favorites_count!",
                author.username author_username,
                author.bio author_bio,
                author.image author_image,
                false "following_author!"
            from updated_article
            inner join users author on author.user_id = $7
        "#,
        new_slug,
        req.article.title,
        req.article.description,
        req.article.body,
        req.article.tag_list.as_deref(),
        article_meta.article_id,
        auth_user.user_id
    )
    .fetch_one(&mut *tx)
    .await
    .map_err(Error::from_sqlx)?
    .into_article();

    tx.commit().await?;

    Ok(Json(ArticleBody { article }))
}

pub async fn delete_article(
    auth_user: AuthUser,
    State(state): State<AppState>,
    Path(slug): Path<String>,
) -> Result<StatusCode, Error> {
    let result = sqlx::query!(
        r#"
        with deleted_article as (
            delete from articles
            where slug = $1 and user_id = $2
            returning 1
        )
        select
            exists(select 1 from articles where slug = $1) "existed!",
            exists(select 1 from deleted_article) "deleted!"
    "#,
        slug,
        auth_user.user_id
    )
    .fetch_one(&state.db)
    .await?;

    if result.deleted {
        Ok(StatusCode::NO_CONTENT)
    } else if result.existed {
        Err(Error::Forbidden)
    } else {
        Err(Error::NotFound)
    }
}

pub async fn get_article(
    maybe_auth_user: MaybeAuthUser,
    State(state): State<AppState>,
    Path(slug): Path<String>,
) -> Result<Json<ArticleBody>, Error> {
    let article = sqlx::query_as!(
        ArticleFromQuery,
        r#"
        select
            slug,
            title,
            description,
            body,
            tag_list,
            articles.created_at,
            articles.updated_at,
            exists(
                select 1 from favorites
                where user_id = $1 and article_id = articles.article_id
            ) "favorited!",
            coalesce(
                (select count(*) from favorites fav where fav.article_id = articles.article_id),
                0
            ) "favorites_count!",
            author.username author_username,
            author.bio author_bio,
            author.image author_image,
            exists(
                select 1 from follow
                where follower_user_id = $1 and following_user_id = author.user_id
            ) "following_author!"
        from articles
        inner join users author using (user_id)
        where slug = $2
    "#,
        maybe_auth_user.user_id(),
        slug
    )
    .fetch_optional(&state.db)
    .await?
    .ok_or(Error::NotFound)?
    .into_article();

    Ok(Json(ArticleBody { article }))
}

pub async fn favorite_article(
    auth_user: AuthUser,
    State(state): State<AppState>,
    Path(slug): Path<String>,
) -> Result<Json<ArticleBody>, Error> {
    let article = sqlx::query_as!(
        ArticleFromQuery,
        r#"
        with inserted_favorite as (
            insert into favorites (article_id, user_id)
            select article_id, $2
            from articles
            where slug = $1
            on conflict do nothing
        )
        select
            slug,
            title,
            description,
            body,
            tag_list,
            articles.created_at,
            articles.updated_at,
            true "favorited!",
            coalesce(
                (select count(*) from favorites fav where fav.article_id = articles.article_id),
                0
            ) "favorites_count!",
            author.username author_username,
            author.bio author_bio,
            author.image author_image,
            exists(
                select 1 from follow
                where follower_user_id = $2 and following_user_id = author.user_id
            ) "following_author!"
        from articles
        inner join users author using (user_id)
        where slug = $1
    "#,
        slug,
        auth_user.user_id
    )
    .fetch_optional(&state.db)
    .await?
    .ok_or(Error::NotFound)?
    .into_article();

    Ok(Json(ArticleBody { article }))
}

pub async fn unfavorite_article(
    auth_user: AuthUser,
    State(state): State<AppState>,
    Path(slug): Path<String>,
) -> Result<Json<ArticleBody>, Error> {
    let article = sqlx::query_as!(
        ArticleFromQuery,
        r#"
        with deleted_favorite as (
            delete from favorites
            where article_id = (
                select article_id
                from articles
                where slug = $1
            )
            and user_id = $2
        )
        select
            slug,
            title,
            description,
            body,
            tag_list,
            articles.created_at,
            articles.updated_at,
            true "favorited!",
            coalesce(
                (select count(*) from favorites fav where fav.article_id = articles.article_id),
                0
            ) "favorites_count!",
            author.username author_username,
            author.bio author_bio,
            author.image author_image,
            exists(
                select 1 from follow
                where follower_user_id = $2 and following_user_id = author.user_id
            ) "following_author!"
        from articles
        inner join users author using (user_id)
        where slug = $1
    "#,
        slug,
        auth_user.user_id
    )
    .fetch_optional(&state.db)
    .await?
    .ok_or(Error::NotFound)?
    .into_article();

    Ok(Json(ArticleBody { article }))
}

pub async fn get_tags(State(state): State<AppState>) -> Result<Json<TagsBody>, Error> {
    let tags = sqlx::query_scalar!(
        r#"
        select distinct tag "tag!"
        from articles, unnest (articles.tag_list) tags(tag)
        order by tag
    "#
    )
    .fetch_all(&state.db)
    .await?;

    Ok(Json(TagsBody { tags }))
}

pub async fn list_articles(
    maybe_auth_user: MaybeAuthUser,
    State(state): State<AppState>,
    query: Query<ListArticlesQuery>,
) -> Result<Json<MultipleArticlesBody>, Error> {
    let articles: Vec<_> = sqlx::query_as!(
        ArticleFromQuery,
        r#"
        select
                slug,
                title,
                description,
                body,
                tag_list,
                articles.created_at,
                articles.updated_at,

                exists (
                    select 1
                    from favorites f
                    where f.article_id = articles.article_id
                    and f.user_id = $1
                ) as "favorited!",

                (
                    select count(*)
                    from favorites f
                    where f.article_id = articles.article_id
                ) as "favorites_count!",

                author.username as author_username,
                author.bio as author_bio,
                author.image as author_image,

                exists (
                    select 1
                    from follow f
                    where f.following_user_id = author.user_id
                    and f.follower_user_id = $1
                ) as "following_author!"

            from articles
            inner join users author using (user_id)

            where
                ($2::text is null or tag_list @> array[$2])
                and ($3::text is null or author.username = $3)
                and (
                    $4::text is null
                    or exists (
                        select 1
                        from users u
                        inner join favorites f on f.user_id = u.user_id
                        where u.username = $4
                        and f.article_id = articles.article_id
                    )
                )

            order by articles.created_at desc
            limit $5
            offset $6
        "#,
        maybe_auth_user.user_id(),
        query.tag,
        query.author,
        query.favorited,
        query.limit.unwrap_or(20),
        query.offset.unwrap_or(0)
    )
    .fetch(&state.db)
    .map_ok(ArticleFromQuery::into_article)
    .try_collect()
    .await?;

    Ok(Json(MultipleArticlesBody {
        articles_count: articles.len(),
        articles,
    }))
}

pub async fn feed_articles(
    maybe_auth_user: MaybeAuthUser,
    State(state): State<AppState>,
    query: Query<FeedArticlesQuery>,
) -> Result<Json<MultipleArticlesBody>, Error> {
    let articles: Vec<_> = sqlx::query_as!(
        ArticleFromQuery,
        r#"
            select
                article.slug,
                article.title,
                article.description,
                article.body,
                article.tag_list,
                article.created_at,
                article.updated_at,
                true as "favorited!",
                (
                    select count(*)
                    from favorites fav
                    where fav.article_id = article.article_id
                ) as "favorites_count!",

                author.username as author_username,
                author.bio as author_bio,
                author.image as author_image,
                true as "following_author!"
            from follow
            inner join articles article
                on article.user_id = follow.following_user_id
            inner join users author
                on author.user_id = article.user_id
            where follow.follower_user_id = $1
            order by article.created_at desc
            limit $2
            offset $3
        "#,
        maybe_auth_user.user_id(),
        query.limit.unwrap_or(20),
        query.offset.unwrap_or(0)
    )
    .fetch(&state.db)
    .map_ok(ArticleFromQuery::into_article)
    .try_collect()
    .await?;

    Ok(Json(MultipleArticlesBody {
        articles_count: articles.len(),
        articles,
    }))
}
