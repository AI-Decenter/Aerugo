use axum::{
    extract::{Path, State},
    Json,
};
use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    models::user::{CreateUserRequest, UpdateUserRequest, User},
    utils::errors::{AppError, Result},
};

// Dummy auth check
fn check_auth() -> Result<()> {
    // For testing Auth error
    Err(AppError::Auth("Invalid credentials".into()))
}

pub async fn create_user(
    State(pool): State<PgPool>,
    Json(payload): Json<CreateUserRequest>,
) -> Result<Json<User>> {
    // Example Validation error
    if payload.name.is_empty() {
        return Err(AppError::Validation("Name cannot be empty".into()));
    }

    let user = sqlx::query_as::<_, User>(
        r#"
        INSERT INTO users (id, name, email, created_at, updated_at)
        VALUES ($1, $2, $3, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)
        RETURNING *
        "#,
    )
    .bind(Uuid::new_v4())
    .bind(&payload.name)
    .bind(&payload.email)
    .fetch_one(&pool)
    .await
    .map_err(|e| AppError::Database(e))?;

    Ok(Json(user))
}

pub async fn get_user(
    State(pool): State<PgPool>,
    Path(id): Path<String>, // use String to test UUID parsing error
) -> Result<Json<User>> {
    // Trigger Auth error for demo
    check_auth()?;

    // Test UUID parsing
    let user_id = Uuid::parse_str(&id)
        .map_err(|_| AppError::Validation("Invalid UUID".into()))?;

    let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = $1")
        .bind(user_id)
        .fetch_optional(&pool)
        .await
        .map_err(|e| AppError::Database(e))?
        .ok_or_else(|| AppError::NotFound("User not found".into()))?;

    Ok(Json(user))
}

pub async fn update_user(
    State(pool): State<PgPool>,
    Path(id): Path<String>,
    Json(payload): Json<UpdateUserRequest>,
) -> Result<Json<User>> {
    // Test Internal error
    if payload.name.as_deref() == Some("trigger_internal") {
        return Err(AppError::Internal("Simulated internal error".into()));
    }

    let user_id = Uuid::parse_str(&id)
        .map_err(|_| AppError::Validation("Invalid UUID".into()))?;

    let user = sqlx::query_as::<_, User>(
        r#"
        UPDATE users
        SET
            name = COALESCE($1, name),
            email = COALESCE($2, email),
            updated_at = CURRENT_TIMESTAMP
        WHERE id = $3
        RETURNING *
        "#,
    )
    .bind(payload.name)
    .bind(payload.email)
    .bind(user_id)
    .fetch_optional(&pool)
    .await
    .map_err(|e| AppError::Database(e))?
    .ok_or_else(|| AppError::NotFound("User not found".into()))?;

    Ok(Json(user))
}

pub async fn delete_user(
    State(pool): State<PgPool>,
    Path(id): Path<String>,
) -> Result<Json<()>> {
    let user_id = Uuid::parse_str(&id)
        .map_err(|_| AppError::Validation("Invalid UUID".into()))?;

    let result = sqlx::query("DELETE FROM users WHERE id = $1")
        .bind(user_id)
        .execute(&pool)
        .await
        .map_err(|e| AppError::Database(e))?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound("User not found".into()));
    }

    Ok(Json(()))
}
