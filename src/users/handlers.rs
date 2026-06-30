use anyhow::Context;
use argon2::password_hash::{SaltString, rand_core::OsRng};
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use axum::extract::{Json, State};

use crate::error::Error;
use crate::extractor::AuthUser;
use crate::state::AppState;
use crate::users::dto::{CreateUser, LoginUser, UpdateUser, User, UserBody};

async fn hash_password(password: String) -> Result<String, anyhow::Error> {
    tokio::task::spawn_blocking(move || {
        let salt = SaltString::generate(&mut OsRng);

        Argon2::default()
            .hash_password(password.as_bytes(), &salt)
            .map(|hash| hash.to_string())
            .context("failed to hash password")
    })
    .await
    .context("panic while hashing password")?
}

async fn verify_password(password: String, password_hash: String) -> Result<(), Error> {
    tokio::task::spawn_blocking(move || -> Result<(), Error> {
        let hash = PasswordHash::new(&password_hash)
            .map_err(|e| anyhow::anyhow!("invalid password hash {e}"))?;

        Argon2::default()
            .verify_password(password.as_bytes(), &hash)
            .map_err(|e| match e {
                argon2::password_hash::Error::Password => Error::Unauthorized,
                _ => anyhow::anyhow!("failed to verify password hash: {}", e).into(),
            })
    })
    .await
    .map_err(|e| anyhow::anyhow!("panic while verifying password: {e}"))?
}

pub async fn create_user(
    State(state): State<AppState>,
    Json(req): Json<UserBody<CreateUser>>,
) -> Result<Json<UserBody<User>>, Error> {
    let CreateUser {
        username,
        email,
        password,
    } = req.user;

    let hashed_password = hash_password(password).await?;

    let _user_id = sqlx::query_scalar!(
        r#"
        insert into "users" (username, email, password_hash)
        values ($1, $2, $3)
        returning user_id
        "#,
        &username,
        &email,
        &hashed_password
    )
    .fetch_one(&state.db)
    .await
    .map_err(Error::from_sqlx)?;

    Ok(Json(UserBody {
        user: User {
            email,
            token: String::new(),
            username,
            bio: String::new(),
            image: None,
        },
    }))
}

pub async fn get_current_user(
    auth_user: AuthUser,
    State(state): State<AppState>,
) -> Result<Json<UserBody<User>>, Error> {
    let user = sqlx::query!(
        r#"
        select email, username, bio, image from "users" where user_id = $1
    "#,
        auth_user.user_id
    )
    .fetch_one(&state.db)
    .await?;

    Ok(Json(UserBody {
        user: User {
            email: user.email,
            token: auth_user.to_jwt(&state),
            username: user.username,
            bio: user.bio,
            image: user.image,
        },
    }))
}

pub async fn login_user(
    State(state): State<AppState>,
    Json(req): Json<UserBody<LoginUser>>,
) -> Result<Json<UserBody<User>>, Error> {
    let LoginUser { email, password } = req.user;

    let user = sqlx::query!(
        r#"
            select user_id, email, username, bio, image, password_hash
            from "users" where email = $1
        "#,
        &email,
    )
    .fetch_optional(&state.db)
    .await?
    .ok_or(Error::validation("email", "does not exist"))?;

    verify_password(password, user.password_hash).await?;

    Ok(Json(UserBody {
        user: User {
            email,
            token: AuthUser {
                user_id: user.user_id,
            }
            .to_jwt(&state),
            username: user.username,
            bio: user.bio,
            image: user.image,
        },
    }))
}

pub async fn update_user(
    auth_user: AuthUser,
    State(state): State<AppState>,
    Json(req): Json<UserBody<UpdateUser>>,
) -> Result<Json<UserBody<User>>, Error> {
    if req.user == UpdateUser::default() {
        return get_current_user(auth_user, State(state)).await;
    }

    let password_hash = if let Some(password) = req.user.password {
        Some(hash_password(password).await?)
    } else {
        None
    };

    let user = sqlx::query!(
        r#"
        update "users"
        set email = coalesce($1, "users".email),
            username = coalesce($2, "users".username),
            password_hash = coalesce($3, "users".password_hash),
            bio = coalesce($4, "users".bio),
            image = coalesce($5, "users".image)
        where user_id = $6
        returning email, username, bio, image
    "#,
        req.user.email,
        req.user.username,
        password_hash,
        req.user.bio,
        req.user.image,
        auth_user.user_id
    )
    .fetch_one(&state.db)
    .await
    .map_err(Error::from_sqlx)?;

    Ok(Json(UserBody {
        user: User {
            email: user.email,
            token: auth_user.to_jwt(&state),
            username: user.username,
            bio: user.bio,
            image: user.image,
        },
    }))
}
