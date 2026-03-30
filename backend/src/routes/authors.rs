use axum::extract::{Path, State};
use axum::routing::get;
use axum::{Json, Router};
use sqlx::SqlitePool;
use types::{Author, AuthorSummary, AuthorWithBooks, Book, CreateAuthor, UpdateAuthor};

use crate::errors::AppError;

pub fn router() -> Router<SqlitePool> {
    Router::new()
        .route("/api/authors", get(list_authors).post(create_author))
        .route(
            "/api/authors/{id}",
            get(get_author).put(update_author).delete(delete_author),
        )
}

async fn list_authors(State(pool): State<SqlitePool>) -> Result<Json<Vec<AuthorSummary>>, AppError> {
    let rows = sqlx::query_as::<_, AuthorSummaryRow>(
        "SELECT a.id, a.name, a.bio, a.created_at, COUNT(b.id) as book_count \
         FROM authors a LEFT JOIN books b ON b.author_id = a.id \
         GROUP BY a.id ORDER BY a.name",
    )
    .fetch_all(&pool)
    .await?;

    let authors = rows
        .into_iter()
        .map(|r| AuthorSummary {
            id: r.id,
            name: r.name,
            bio: r.bio,
            book_count: r.book_count,
            created_at: r.created_at,
        })
        .collect();

    Ok(Json(authors))
}

#[derive(sqlx::FromRow)]
struct AuthorSummaryRow {
    id: i64,
    name: String,
    bio: Option<String>,
    book_count: i64,
    created_at: String,
}

async fn create_author(
    State(pool): State<SqlitePool>,
    Json(input): Json<CreateAuthor>,
) -> Result<(axum::http::StatusCode, Json<Author>), AppError> {
    if input.name.trim().is_empty() {
        return Err(AppError::BadRequest("Author name is required".to_string()));
    }

    let author = sqlx::query_as::<_, Author>(
        "INSERT INTO authors (name, bio) VALUES (?, ?) RETURNING id, name, bio, created_at",
    )
    .bind(input.name.trim())
    .bind(input.bio.as_deref().map(str::trim))
    .fetch_one(&pool)
    .await?;

    Ok((axum::http::StatusCode::CREATED, Json(author)))
}

async fn get_author(
    State(pool): State<SqlitePool>,
    Path(id): Path<i64>,
) -> Result<Json<AuthorWithBooks>, AppError> {
    let author = sqlx::query_as::<_, Author>(
        "SELECT id, name, bio, created_at FROM authors WHERE id = ?",
    )
    .bind(id)
    .fetch_optional(&pool)
    .await?
    .ok_or_else(|| AppError::NotFound(format!("Author with id {id} not found")))?;

    let books = sqlx::query_as::<_, Book>(
        "SELECT id, title, author_id, isbn, published_year, genre, rating, cover_url, description, created_at FROM books WHERE author_id = ? ORDER BY title",
    )
    .bind(id)
    .fetch_all(&pool)
    .await?;

    Ok(Json(AuthorWithBooks { author, books }))
}

async fn update_author(
    State(pool): State<SqlitePool>,
    Path(id): Path<i64>,
    Json(input): Json<UpdateAuthor>,
) -> Result<Json<Author>, AppError> {
    let existing = sqlx::query_as::<_, Author>(
        "SELECT id, name, bio, created_at FROM authors WHERE id = ?",
    )
    .bind(id)
    .fetch_optional(&pool)
    .await?
    .ok_or_else(|| AppError::NotFound(format!("Author with id {id} not found")))?;

    let name = input
        .name
        .as_deref()
        .map(str::trim)
        .unwrap_or(&existing.name);
    if name.is_empty() {
        return Err(AppError::BadRequest("Author name cannot be empty".to_string()));
    }

    let bio = match &input.bio {
        Some(b) => Some(b.trim().to_string()),
        None => existing.bio.clone(),
    };

    let author = sqlx::query_as::<_, Author>(
        "UPDATE authors SET name = ?, bio = ? WHERE id = ? RETURNING id, name, bio, created_at",
    )
    .bind(name)
    .bind(bio)
    .bind(id)
    .fetch_one(&pool)
    .await?;

    Ok(Json(author))
}

async fn delete_author(
    State(pool): State<SqlitePool>,
    Path(id): Path<i64>,
) -> Result<axum::http::StatusCode, AppError> {
    let result = sqlx::query("DELETE FROM authors WHERE id = ?")
        .bind(id)
        .execute(&pool)
        .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound(format!("Author with id {id} not found")));
    }

    Ok(axum::http::StatusCode::NO_CONTENT)
}
