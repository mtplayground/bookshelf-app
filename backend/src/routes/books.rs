use axum::extract::{Path, Query, State};
use axum::routing::get;
use axum::{Json, Router};
use sqlx::SqlitePool;
use types::{Book, BookQuery, BookStats, BookWithAuthor, CreateBook, GenreCount, UpdateBook};

use crate::errors::AppError;

pub fn router() -> Router<SqlitePool> {
    Router::new()
        .route("/api/books", get(list_books).post(create_book))
        .route("/api/books/stats", get(book_stats))
        .route(
            "/api/books/{id}",
            get(get_book).put(update_book).delete(delete_book),
        )
}

async fn list_books(
    State(pool): State<SqlitePool>,
    Query(params): Query<BookQuery>,
) -> Result<Json<Vec<BookWithAuthor>>, AppError> {
    let mut sql = String::from(
        "SELECT b.id, b.title, b.author_id, b.isbn, b.published_year, b.genre, \
         b.rating, b.cover_url, b.description, b.created_at, a.name as author_name \
         FROM books b JOIN authors a ON b.author_id = a.id WHERE 1=1",
    );
    let mut binds: Vec<String> = Vec::new();

    if let Some(ref search) = params.search {
        let s = search.trim();
        if !s.is_empty() {
            sql.push_str(" AND (b.title LIKE ? OR a.name LIKE ? OR b.isbn LIKE ?)");
            let pattern = format!("%{s}%");
            binds.push(pattern.clone());
            binds.push(pattern.clone());
            binds.push(pattern);
        }
    }

    if let Some(ref genre) = params.genre {
        let g = genre.trim();
        if !g.is_empty() {
            sql.push_str(" AND b.genre = ?");
            binds.push(g.to_string());
        }
    }

    let order = match params.sort.as_deref() {
        Some("title") => " ORDER BY b.title ASC",
        Some("title_desc") => " ORDER BY b.title DESC",
        Some("rating") => " ORDER BY b.rating DESC NULLS LAST",
        Some("year") => " ORDER BY b.published_year DESC NULLS LAST",
        Some("newest") => " ORDER BY b.created_at DESC",
        _ => " ORDER BY b.created_at DESC",
    };
    sql.push_str(order);

    let mut query = sqlx::query_as::<_, BookWithAuthorRow>(&sql);
    for b in &binds {
        query = query.bind(b);
    }

    let rows = query.fetch_all(&pool).await?;

    let books = rows.into_iter().map(|r| r.into_book_with_author()).collect();
    Ok(Json(books))
}

async fn create_book(
    State(pool): State<SqlitePool>,
    Json(input): Json<CreateBook>,
) -> Result<(axum::http::StatusCode, Json<Book>), AppError> {
    validate_book_fields(
        &input.title,
        input.rating,
        input.isbn.as_deref(),
    )?;

    // Verify author exists
    let author_exists = sqlx::query_scalar::<_, i64>("SELECT id FROM authors WHERE id = ?")
        .bind(input.author_id)
        .fetch_optional(&pool)
        .await?;
    if author_exists.is_none() {
        return Err(AppError::BadRequest(format!(
            "Author with id {} not found",
            input.author_id
        )));
    }

    let book = sqlx::query_as::<_, Book>(
        "INSERT INTO books (title, author_id, isbn, published_year, genre, rating, cover_url, description) \
         VALUES (?, ?, ?, ?, ?, ?, ?, ?) \
         RETURNING id, title, author_id, isbn, published_year, genre, rating, cover_url, description, created_at",
    )
    .bind(input.title.trim())
    .bind(input.author_id)
    .bind(input.isbn.as_deref().map(str::trim))
    .bind(input.published_year)
    .bind(input.genre.as_deref().map(str::trim))
    .bind(input.rating)
    .bind(input.cover_url.as_deref().map(str::trim))
    .bind(input.description.as_deref().map(str::trim))
    .fetch_one(&pool)
    .await
    .map_err(|e| match e {
        sqlx::Error::Database(ref db_err) if db_err.message().contains("UNIQUE") => {
            AppError::BadRequest("A book with this ISBN already exists".to_string())
        }
        other => AppError::from(other),
    })?;

    Ok((axum::http::StatusCode::CREATED, Json(book)))
}

async fn get_book(
    State(pool): State<SqlitePool>,
    Path(id): Path<i64>,
) -> Result<Json<BookWithAuthor>, AppError> {
    let row = sqlx::query_as::<_, BookWithAuthorRow>(
        "SELECT b.id, b.title, b.author_id, b.isbn, b.published_year, b.genre, \
         b.rating, b.cover_url, b.description, b.created_at, a.name as author_name \
         FROM books b JOIN authors a ON b.author_id = a.id WHERE b.id = ?",
    )
    .bind(id)
    .fetch_optional(&pool)
    .await?
    .ok_or_else(|| AppError::NotFound(format!("Book with id {id} not found")))?;

    Ok(Json(row.into_book_with_author()))
}

async fn update_book(
    State(pool): State<SqlitePool>,
    Path(id): Path<i64>,
    Json(input): Json<UpdateBook>,
) -> Result<Json<Book>, AppError> {
    let existing = sqlx::query_as::<_, Book>(
        "SELECT id, title, author_id, isbn, published_year, genre, rating, cover_url, description, created_at \
         FROM books WHERE id = ?",
    )
    .bind(id)
    .fetch_optional(&pool)
    .await?
    .ok_or_else(|| AppError::NotFound(format!("Book with id {id} not found")))?;

    let title = input.title.as_deref().map(str::trim).unwrap_or(&existing.title);
    let author_id = input.author_id.unwrap_or(existing.author_id);
    let isbn = match &input.isbn {
        Some(v) => Some(v.trim().to_string()),
        None => existing.isbn.clone(),
    };
    let published_year = match input.published_year {
        Some(v) => Some(v),
        None => existing.published_year,
    };
    let genre = match &input.genre {
        Some(v) => Some(v.trim().to_string()),
        None => existing.genre.clone(),
    };
    let rating = match input.rating {
        Some(v) => Some(v),
        None => existing.rating,
    };
    let cover_url = match &input.cover_url {
        Some(v) => Some(v.trim().to_string()),
        None => existing.cover_url.clone(),
    };
    let description = match &input.description {
        Some(v) => Some(v.trim().to_string()),
        None => existing.description.clone(),
    };

    validate_book_fields(title, rating, isbn.as_deref())?;

    if input.author_id.is_some() {
        let author_exists = sqlx::query_scalar::<_, i64>("SELECT id FROM authors WHERE id = ?")
            .bind(author_id)
            .fetch_optional(&pool)
            .await?;
        if author_exists.is_none() {
            return Err(AppError::BadRequest(format!(
                "Author with id {author_id} not found"
            )));
        }
    }

    let book = sqlx::query_as::<_, Book>(
        "UPDATE books SET title = ?, author_id = ?, isbn = ?, published_year = ?, genre = ?, \
         rating = ?, cover_url = ?, description = ? WHERE id = ? \
         RETURNING id, title, author_id, isbn, published_year, genre, rating, cover_url, description, created_at",
    )
    .bind(title)
    .bind(author_id)
    .bind(&isbn)
    .bind(published_year)
    .bind(&genre)
    .bind(rating)
    .bind(&cover_url)
    .bind(&description)
    .bind(id)
    .fetch_one(&pool)
    .await
    .map_err(|e| match e {
        sqlx::Error::Database(ref db_err) if db_err.message().contains("UNIQUE") => {
            AppError::BadRequest("A book with this ISBN already exists".to_string())
        }
        other => AppError::from(other),
    })?;

    Ok(Json(book))
}

async fn delete_book(
    State(pool): State<SqlitePool>,
    Path(id): Path<i64>,
) -> Result<axum::http::StatusCode, AppError> {
    let result = sqlx::query("DELETE FROM books WHERE id = ?")
        .bind(id)
        .execute(&pool)
        .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound(format!("Book with id {id} not found")));
    }

    Ok(axum::http::StatusCode::NO_CONTENT)
}

async fn book_stats(State(pool): State<SqlitePool>) -> Result<Json<BookStats>, AppError> {
    let total_books = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM books")
        .fetch_one(&pool)
        .await?;

    let total_authors = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM authors")
        .fetch_one(&pool)
        .await?;

    let avg_rating = sqlx::query_scalar::<_, Option<f64>>(
        "SELECT AVG(CAST(rating AS REAL)) FROM books WHERE rating IS NOT NULL",
    )
    .fetch_one(&pool)
    .await?;

    let genre_rows = sqlx::query_as::<_, GenreCountRow>(
        "SELECT genre, COUNT(*) as count FROM books WHERE genre IS NOT NULL AND genre != '' GROUP BY genre ORDER BY count DESC",
    )
    .fetch_all(&pool)
    .await?;

    let books_by_genre = genre_rows
        .into_iter()
        .map(|r| GenreCount {
            genre: r.genre,
            count: r.count,
        })
        .collect();

    Ok(Json(BookStats {
        total_books,
        total_authors,
        avg_rating,
        books_by_genre,
    }))
}

// Internal helpers

fn validate_book_fields(
    title: &str,
    rating: Option<i32>,
    isbn: Option<&str>,
) -> Result<(), AppError> {
    if title.trim().is_empty() {
        return Err(AppError::BadRequest("Book title is required".to_string()));
    }
    if let Some(r) = rating {
        if !(0..=5).contains(&r) {
            return Err(AppError::BadRequest(
                "Rating must be between 0 and 5".to_string(),
            ));
        }
    }
    if let Some(isbn) = isbn {
        let digits: String = isbn.chars().filter(|c| c.is_ascii_digit()).collect();
        if !isbn.trim().is_empty() && digits.len() != 10 && digits.len() != 13 {
            return Err(AppError::BadRequest(
                "ISBN must be 10 or 13 digits".to_string(),
            ));
        }
    }
    Ok(())
}

// Row types for JOINed queries

#[derive(sqlx::FromRow)]
struct BookWithAuthorRow {
    id: i64,
    title: String,
    author_id: i64,
    isbn: Option<String>,
    published_year: Option<i32>,
    genre: Option<String>,
    rating: Option<i32>,
    cover_url: Option<String>,
    description: Option<String>,
    created_at: String,
    author_name: String,
}

impl BookWithAuthorRow {
    fn into_book_with_author(self) -> BookWithAuthor {
        BookWithAuthor {
            book: Book {
                id: self.id,
                title: self.title,
                author_id: self.author_id,
                isbn: self.isbn,
                published_year: self.published_year,
                genre: self.genre,
                rating: self.rating,
                cover_url: self.cover_url,
                description: self.description,
                created_at: self.created_at,
            },
            author_name: self.author_name,
        }
    }
}

#[derive(sqlx::FromRow)]
struct GenreCountRow {
    genre: String,
    count: i64,
}
