use axum::body::Body;
use axum::http::{Request, StatusCode};
use axum::Router;
use http_body_util::BodyExt;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use tower::ServiceExt;
use types::*;

async fn setup() -> Router {
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect_with(
            SqliteConnectOptions::new()
                .filename(":memory:")
                .create_if_missing(true)
                .foreign_keys(true),
        )
        .await
        .unwrap();

    sqlx::migrate!("./migrations").run(&pool).await.unwrap();
    backend::build_app(pool)
}

// --- helpers ---

async fn get_json<T: serde::de::DeserializeOwned>(app: &Router, uri: &str) -> (StatusCode, T) {
    let resp = app
        .clone()
        .oneshot(Request::get(uri).body(Body::empty()).unwrap())
        .await
        .unwrap();
    let status = resp.status();
    let body = resp.into_body().collect().await.unwrap().to_bytes();
    (status, serde_json::from_slice(&body).unwrap())
}

async fn get_status(app: &Router, uri: &str) -> StatusCode {
    app.clone()
        .oneshot(Request::get(uri).body(Body::empty()).unwrap())
        .await
        .unwrap()
        .status()
}

async fn post_json<B: serde::Serialize, T: serde::de::DeserializeOwned>(
    app: &Router,
    uri: &str,
    body: &B,
) -> (StatusCode, T) {
    let resp = app
        .clone()
        .oneshot(
            Request::post(uri)
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_vec(body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();
    let status = resp.status();
    let body = resp.into_body().collect().await.unwrap().to_bytes();
    (status, serde_json::from_slice(&body).unwrap())
}

async fn post_status<B: serde::Serialize>(app: &Router, uri: &str, body: &B) -> StatusCode {
    app.clone()
        .oneshot(
            Request::post(uri)
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_vec(body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap()
        .status()
}

async fn put_json<B: serde::Serialize, T: serde::de::DeserializeOwned>(
    app: &Router,
    uri: &str,
    body: &B,
) -> (StatusCode, T) {
    let resp = app
        .clone()
        .oneshot(
            Request::put(uri)
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_vec(body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();
    let status = resp.status();
    let body = resp.into_body().collect().await.unwrap().to_bytes();
    (status, serde_json::from_slice(&body).unwrap())
}

async fn delete_req(app: &Router, uri: &str) -> StatusCode {
    app.clone()
        .oneshot(Request::delete(uri).body(Body::empty()).unwrap())
        .await
        .unwrap()
        .status()
}

async fn create_test_author(app: &Router) -> Author {
    let input = CreateAuthor {
        name: "Test Author".to_string(),
        bio: Some("A bio".to_string()),
    };
    let (_, author): (_, Author) = post_json(app, "/api/authors", &input).await;
    author
}

async fn create_test_book(app: &Router, author_id: i64) -> Book {
    let input = CreateBook {
        title: "Test Book".to_string(),
        author_id,
        isbn: Some("9781234567890".to_string()),
        published_year: Some(2024),
        genre: Some("Fiction".to_string()),
        rating: Some(4),
        description: Some("A great book".to_string()),
        ..Default::default()
    };
    let (_, book): (_, Book) = post_json(app, "/api/books", &input).await;
    book
}

// --- tests ---

#[tokio::test]
async fn health_check() {
    let app = setup().await;
    let (status, health): (_, HealthResponse) = get_json(&app, "/api/health").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(health.status, "ok");
}

#[tokio::test]
async fn author_crud() {
    let app = setup().await;

    // Create
    let input = CreateAuthor {
        name: "Jane Austen".to_string(),
        bio: Some("English novelist".to_string()),
    };
    let (status, author): (_, Author) = post_json(&app, "/api/authors", &input).await;
    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(author.name, "Jane Austen");
    assert_eq!(author.bio.as_deref(), Some("English novelist"));

    // List
    let (status, authors): (_, Vec<AuthorSummary>) = get_json(&app, "/api/authors").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(authors.len(), 1);
    assert_eq!(authors[0].name, "Jane Austen");
    assert_eq!(authors[0].book_count, 0);

    // Get
    let (status, detail): (_, AuthorWithBooks) =
        get_json(&app, &format!("/api/authors/{}", author.id)).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(detail.author.name, "Jane Austen");
    assert!(detail.books.is_empty());

    // Update
    let update = UpdateAuthor {
        name: Some("J. Austen".to_string()),
        bio: None,
    };
    let (status, updated): (_, Author) =
        put_json(&app, &format!("/api/authors/{}", author.id), &update).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(updated.name, "J. Austen");
    assert_eq!(updated.bio.as_deref(), Some("English novelist")); // unchanged

    // Delete
    let status = delete_req(&app, &format!("/api/authors/{}", author.id)).await;
    assert_eq!(status, StatusCode::NO_CONTENT);

    // Get after delete
    let status = get_status(&app, &format!("/api/authors/{}", author.id)).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn book_crud() {
    let app = setup().await;
    let author = create_test_author(&app).await;

    // Create
    let input = CreateBook {
        title: "Pride and Prejudice".to_string(),
        author_id: author.id,
        isbn: Some("9780141439518".to_string()),
        published_year: Some(1813),
        genre: Some("Fiction".to_string()),
        rating: Some(5),
        description: Some("A classic".to_string()),
        ..Default::default()
    };
    let (status, book): (_, Book) = post_json(&app, "/api/books", &input).await;
    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(book.title, "Pride and Prejudice");
    assert_eq!(book.rating, Some(5));

    // List
    let (status, books): (_, Vec<BookWithAuthor>) = get_json(&app, "/api/books").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(books.len(), 1);
    assert_eq!(books[0].author_name, "Test Author");

    // Get
    let (status, detail): (_, BookWithAuthor) =
        get_json(&app, &format!("/api/books/{}", book.id)).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(detail.book.title, "Pride and Prejudice");
    assert_eq!(detail.author_name, "Test Author");

    // Update
    let update = UpdateBook {
        title: Some("P&P".to_string()),
        rating: Some(4),
        ..Default::default()
    };
    let (status, updated): (_, Book) =
        put_json(&app, &format!("/api/books/{}", book.id), &update).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(updated.title, "P&P");
    assert_eq!(updated.rating, Some(4));

    // Delete
    let status = delete_req(&app, &format!("/api/books/{}", book.id)).await;
    assert_eq!(status, StatusCode::NO_CONTENT);

    let status = get_status(&app, &format!("/api/books/{}", book.id)).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn validation_empty_author_name() {
    let app = setup().await;
    let input = CreateAuthor {
        name: "  ".to_string(),
        bio: None,
    };
    let status = post_status(&app, "/api/authors", &input).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn validation_empty_book_title() {
    let app = setup().await;
    let author = create_test_author(&app).await;
    let input = CreateBook {
        title: "".to_string(),
        author_id: author.id,
        ..Default::default()
    };
    let status = post_status(&app, "/api/books", &input).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn validation_invalid_rating() {
    let app = setup().await;
    let author = create_test_author(&app).await;
    let input = CreateBook {
        title: "Test".to_string(),
        author_id: author.id,
        rating: Some(6),
        ..Default::default()
    };
    let status = post_status(&app, "/api/books", &input).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn validation_invalid_isbn() {
    let app = setup().await;
    let author = create_test_author(&app).await;
    let input = CreateBook {
        title: "Test".to_string(),
        author_id: author.id,
        isbn: Some("12345".to_string()),
        ..Default::default()
    };
    let status = post_status(&app, "/api/books", &input).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn validation_nonexistent_author() {
    let app = setup().await;
    let input = CreateBook {
        title: "Test".to_string(),
        author_id: 9999,
        ..Default::default()
    };
    let status = post_status(&app, "/api/books", &input).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn duplicate_isbn_rejected() {
    let app = setup().await;
    let author = create_test_author(&app).await;

    let book1 = CreateBook {
        title: "Book 1".to_string(),
        author_id: author.id,
        isbn: Some("9781234567890".to_string()),
        ..Default::default()
    };
    post_json::<_, Book>(&app, "/api/books", &book1).await;

    let book2 = CreateBook {
        title: "Book 2".to_string(),
        author_id: author.id,
        isbn: Some("9781234567890".to_string()),
        ..Default::default()
    };
    let status = post_status(&app, "/api/books", &book2).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn book_search() {
    let app = setup().await;
    let author = create_test_author(&app).await;

    for title in ["The Great Gatsby", "Great Expectations", "To Kill a Mockingbird"] {
        let input = CreateBook {
            title: title.to_string(),
            author_id: author.id,
            ..Default::default()
        };
        post_json::<_, Book>(&app, "/api/books", &input).await;
    }

    let (_, results): (_, Vec<BookWithAuthor>) = get_json(&app, "/api/books?search=Great").await;
    assert_eq!(results.len(), 2);
}

#[tokio::test]
async fn book_filter_by_genre() {
    let app = setup().await;
    let author = create_test_author(&app).await;

    let fiction = CreateBook {
        title: "Fiction Book".to_string(),
        author_id: author.id,
        genre: Some("Fiction".to_string()),
        ..Default::default()
    };
    let science = CreateBook {
        title: "Science Book".to_string(),
        author_id: author.id,
        genre: Some("Science".to_string()),
        ..Default::default()
    };
    post_json::<_, Book>(&app, "/api/books", &fiction).await;
    post_json::<_, Book>(&app, "/api/books", &science).await;

    let (_, results): (_, Vec<BookWithAuthor>) =
        get_json(&app, "/api/books?genre=Fiction").await;
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].book.title, "Fiction Book");
}

#[tokio::test]
async fn book_sort_by_title() {
    let app = setup().await;
    let author = create_test_author(&app).await;

    for title in ["Bravo", "Alpha", "Charlie"] {
        let input = CreateBook {
            title: title.to_string(),
            author_id: author.id,
            ..Default::default()
        };
        post_json::<_, Book>(&app, "/api/books", &input).await;
    }

    let (_, results): (_, Vec<BookWithAuthor>) = get_json(&app, "/api/books?sort=title").await;
    assert_eq!(results[0].book.title, "Alpha");
    assert_eq!(results[1].book.title, "Bravo");
    assert_eq!(results[2].book.title, "Charlie");
}

#[tokio::test]
async fn cascade_delete_author_removes_books() {
    let app = setup().await;
    let author = create_test_author(&app).await;
    create_test_book(&app, author.id).await;

    // Verify book exists
    let (_, books): (_, Vec<BookWithAuthor>) = get_json(&app, "/api/books").await;
    assert_eq!(books.len(), 1);

    // Delete author
    let status = delete_req(&app, &format!("/api/authors/{}", author.id)).await;
    assert_eq!(status, StatusCode::NO_CONTENT);

    // Books should be gone
    let (_, books): (_, Vec<BookWithAuthor>) = get_json(&app, "/api/books").await;
    assert_eq!(books.len(), 0);
}

#[tokio::test]
async fn stats_endpoint() {
    let app = setup().await;
    let author = create_test_author(&app).await;
    create_test_book(&app, author.id).await;

    let (status, stats): (_, BookStats) = get_json(&app, "/api/books/stats").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(stats.total_books, 1);
    assert_eq!(stats.total_authors, 1);
    assert!(stats.avg_rating.is_some());
    assert_eq!(stats.books_by_genre.len(), 1);
    assert_eq!(stats.top_rated.len(), 1);
    assert_eq!(stats.most_prolific.len(), 1);
    assert_eq!(stats.recently_added.len(), 1);
}

#[tokio::test]
async fn not_found_for_missing_resources() {
    let app = setup().await;
    assert_eq!(get_status(&app, "/api/authors/999").await, StatusCode::NOT_FOUND);
    assert_eq!(get_status(&app, "/api/books/999").await, StatusCode::NOT_FOUND);
    assert_eq!(delete_req(&app, "/api/authors/999").await, StatusCode::NOT_FOUND);
    assert_eq!(delete_req(&app, "/api/books/999").await, StatusCode::NOT_FOUND);
}
