#![allow(dead_code)]
use gloo_net::http::Request;
use types::*;

pub async fn fetch_books(query: &BookQuery) -> Result<Vec<BookWithAuthor>, String> {
    let mut url = "/api/books".to_string();
    let mut params = Vec::new();
    if let Some(ref search) = query.search {
        if !search.is_empty() {
            params.push(format!("search={search}"));
        }
    }
    if let Some(ref genre) = query.genre {
        if !genre.is_empty() {
            params.push(format!("genre={genre}"));
        }
    }
    if let Some(ref sort) = query.sort {
        if !sort.is_empty() {
            params.push(format!("sort={sort}"));
        }
    }
    if !params.is_empty() {
        url.push('?');
        url.push_str(&params.join("&"));
    }

    let resp = Request::get(&url).send().await.map_err(|e| e.to_string())?;
    parse_response(resp).await
}

pub async fn fetch_book(id: i64) -> Result<BookWithAuthor, String> {
    let resp = Request::get(&format!("/api/books/{id}"))
        .send()
        .await
        .map_err(|e| e.to_string())?;
    parse_response(resp).await
}

pub async fn create_book(book: &CreateBook) -> Result<Book, String> {
    let resp = Request::post("/api/books")
        .json(book)
        .map_err(|e| e.to_string())?
        .send()
        .await
        .map_err(|e| e.to_string())?;
    parse_response(resp).await
}

pub async fn update_book(id: i64, book: &UpdateBook) -> Result<Book, String> {
    let resp = Request::put(&format!("/api/books/{id}"))
        .json(book)
        .map_err(|e| e.to_string())?
        .send()
        .await
        .map_err(|e| e.to_string())?;
    parse_response(resp).await
}

pub async fn delete_book(id: i64) -> Result<(), String> {
    let resp = Request::delete(&format!("/api/books/{id}"))
        .send()
        .await
        .map_err(|e| e.to_string())?;
    if !resp.ok() {
        return Err(extract_error(resp).await);
    }
    Ok(())
}

pub async fn fetch_authors() -> Result<Vec<AuthorSummary>, String> {
    let resp = Request::get("/api/authors")
        .send()
        .await
        .map_err(|e| e.to_string())?;
    parse_response(resp).await
}

pub async fn fetch_author(id: i64) -> Result<AuthorWithBooks, String> {
    let resp = Request::get(&format!("/api/authors/{id}"))
        .send()
        .await
        .map_err(|e| e.to_string())?;
    parse_response(resp).await
}

pub async fn create_author(author: &CreateAuthor) -> Result<Author, String> {
    let resp = Request::post("/api/authors")
        .json(author)
        .map_err(|e| e.to_string())?
        .send()
        .await
        .map_err(|e| e.to_string())?;
    parse_response(resp).await
}

pub async fn update_author(id: i64, author: &UpdateAuthor) -> Result<Author, String> {
    let resp = Request::put(&format!("/api/authors/{id}"))
        .json(author)
        .map_err(|e| e.to_string())?
        .send()
        .await
        .map_err(|e| e.to_string())?;
    parse_response(resp).await
}

pub async fn delete_author(id: i64) -> Result<(), String> {
    let resp = Request::delete(&format!("/api/authors/{id}"))
        .send()
        .await
        .map_err(|e| e.to_string())?;
    if !resp.ok() {
        return Err(extract_error(resp).await);
    }
    Ok(())
}

pub async fn fetch_book_stats() -> Result<BookStats, String> {
    let resp = Request::get("/api/books/stats")
        .send()
        .await
        .map_err(|e| e.to_string())?;
    parse_response(resp).await
}

async fn parse_response<T: serde::de::DeserializeOwned>(
    resp: gloo_net::http::Response,
) -> Result<T, String> {
    if !resp.ok() {
        return Err(extract_error(resp).await);
    }
    resp.json::<T>().await.map_err(|e| e.to_string())
}

async fn extract_error(resp: gloo_net::http::Response) -> String {
    resp.json::<serde_json::Value>()
        .await
        .ok()
        .and_then(|v| v.get("error").and_then(|e| e.as_str()).map(String::from))
        .unwrap_or_else(|| format!("Request failed with status {}", resp.status()))
}
