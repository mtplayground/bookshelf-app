use serde::{Deserialize, Serialize};

// Health

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct HealthResponse {
    pub status: String,
}

// Authors

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[cfg_attr(feature = "backend", derive(sqlx::FromRow))]
pub struct Author {
    pub id: i64,
    pub name: String,
    pub bio: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateAuthor {
    pub name: String,
    pub bio: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateAuthor {
    pub name: Option<String>,
    pub bio: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthorWithBooks {
    #[serde(flatten)]
    pub author: Author,
    pub books: Vec<Book>,
}

// Books

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[cfg_attr(feature = "backend", derive(sqlx::FromRow))]
pub struct Book {
    pub id: i64,
    pub title: String,
    pub author_id: i64,
    pub isbn: Option<String>,
    pub published_year: Option<i32>,
    pub genre: Option<String>,
    pub rating: Option<i32>,
    pub cover_url: Option<String>,
    pub description: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateBook {
    pub title: String,
    pub author_id: i64,
    pub isbn: Option<String>,
    pub published_year: Option<i32>,
    pub genre: Option<String>,
    pub rating: Option<i32>,
    pub cover_url: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateBook {
    pub title: Option<String>,
    pub author_id: Option<i64>,
    pub isbn: Option<String>,
    pub published_year: Option<i32>,
    pub genre: Option<String>,
    pub rating: Option<i32>,
    pub cover_url: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BookWithAuthor {
    #[serde(flatten)]
    pub book: Book,
    pub author_name: String,
}

// Query / Stats

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct BookQuery {
    pub search: Option<String>,
    pub genre: Option<String>,
    pub sort: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenreCount {
    pub genre: String,
    pub count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BookStats {
    pub total_books: i64,
    pub total_authors: i64,
    pub avg_rating: Option<f64>,
    pub books_by_genre: Vec<GenreCount>,
}
