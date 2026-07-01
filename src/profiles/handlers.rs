use axum::{
    Json,
    extract::{Path, State},
};

use crate::{
    error::Error,
    extractor::{AuthUser, MaybeAuthUser},
    profiles::dto::{Profile, ProfileBody},
    state::AppState,
};

pub async fn get_user_profile(
    maybe_auth_user: MaybeAuthUser,
    State(state): State<AppState>,
    Path(username): Path<String>,
) -> Result<Json<ProfileBody>, Error> {
    let profile = sqlx::query_as!(
        Profile,
        r#"
            select
                username,
                bio,
                image,
                exists(
                    select 1 from follow
                    where follower_user_id = $2 and following_user_id = users.user_id
                ) "following!"
            from "users"
            where username = $1
        "#,
        username,
        maybe_auth_user.user_id()
    )
    .fetch_optional(&state.db)
    .await?
    .ok_or(Error::NotFound)?;

    Ok(Json(ProfileBody { profile }))
}

pub async fn follow_user(
    auth_user: AuthUser,
    State(state): State<AppState>,
    Path(username): Path<String>,
) -> Result<Json<ProfileBody>, Error> {
    let mut tx = state.db.begin().await?;

    let user = sqlx::query!(
        r#"select user_id, username, bio, image from users where username = $1"#,
        username
    )
    .fetch_optional(&mut *tx)
    .await?
    .ok_or(Error::NotFound)?;

    let _ = sqlx::query!(
        "insert into follow(following_user_id, follower_user_id) values ($1, $2) \
         on conflict do nothing",
        auth_user.user_id,
        user.user_id
    )
    .execute(&mut *tx)
    .await
    .map_err(Error::from_sqlx)?;

    tx.commit().await?;

    Ok(Json(ProfileBody {
        profile: Profile {
            username: user.username,
            bio: user.bio,
            image: user.image,
            following: true,
        },
    }))
}

pub async fn unfollow_user(
    auth_user: AuthUser,
    State(state): State<AppState>,
    Path(username): Path<String>,
) -> Result<Json<ProfileBody>, Error> {
    let mut tx = state.db.begin().await?;

    let user = sqlx::query!(
        r#"select user_id, username, bio, image from users where username = $1"#,
        username
    )
    .fetch_optional(&mut *tx)
    .await?
    .ok_or(Error::NotFound)?;

    let _ = sqlx::query!(
        "delete from follow where following_user_id = $1 and follower_user_id = $2",
        auth_user.user_id,
        user.user_id
    )
    .execute(&mut *tx)
    .await
    .map_err(Error::from_sqlx)?;

    tx.commit().await?;

    Ok(Json(ProfileBody {
        profile: Profile {
            username: user.username,
            bio: user.bio,
            image: user.image,
            following: false,
        },
    }))
}
