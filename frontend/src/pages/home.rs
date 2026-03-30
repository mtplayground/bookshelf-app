use leptos::prelude::*;
use leptos_router::components::A;
use types::{BookQuery, BookWithAuthor};

use crate::api;

const GENRES: &[&str] = &[
    "Biography",
    "Children",
    "Fantasy",
    "Fiction",
    "History",
    "Horror",
    "Mystery",
    "Non-Fiction",
    "Poetry",
    "Romance",
    "Science",
    "Science Fiction",
    "Self-Help",
];

#[component]
pub fn Home() -> impl IntoView {
    let search = RwSignal::new(String::new());
    let genre = RwSignal::new(String::new());
    let sort = RwSignal::new(String::new());
    let books = RwSignal::new(None::<Result<Vec<BookWithAuthor>, String>>);

    Effect::new(move |_| {
        let s = search.get();
        let g = genre.get();
        let so = sort.get();
        let q = BookQuery {
            search: if s.is_empty() { None } else { Some(s) },
            genre: if g.is_empty() { None } else { Some(g) },
            sort: if so.is_empty() { None } else { Some(so) },
        };
        books.set(None);
        leptos::task::spawn_local(async move {
            books.set(Some(api::fetch_books(&q).await));
        });
    });

    view! {
        <div class="page-header">
            <h1>"Books"</h1>
            <A href="/add-book" attr:class="btn btn-primary">"+ Add Book"</A>
        </div>
        <div class="filter-bar">
            <input
                type="text"
                placeholder="Search books by title, author, or ISBN..."
                prop:value=move || search.get()
                on:input=move |ev| search.set(event_target_value(&ev))
            />
            <select
                prop:value=move || genre.get()
                on:change=move |ev| genre.set(event_target_value(&ev))
            >
                <option value="">"All Genres"</option>
                {GENRES.iter().map(|g| view! {
                    <option value={*g}>{*g}</option>
                }).collect::<Vec<_>>()}
            </select>
            <select
                prop:value=move || sort.get()
                on:change=move |ev| sort.set(event_target_value(&ev))
            >
                <option value="">"Newest First"</option>
                <option value="title">"Title A-Z"</option>
                <option value="title_desc">"Title Z-A"</option>
                <option value="rating">"Highest Rated"</option>
                <option value="year">"Publication Year"</option>
            </select>
        </div>
        <div>
            {move || match books.get() {
                None => view! { <div class="loading">"Loading books..."</div> }.into_any(),
                Some(Err(e)) => view! { <div class="error-msg">{e}</div> }.into_any(),
                Some(Ok(ref list)) if list.is_empty() => view! {
                    <div class="empty-state">
                        <p>"No books found."</p>
                        <A href="/add-book" attr:class="btn btn-primary">"Add your first book"</A>
                    </div>
                }.into_any(),
                Some(Ok(list)) => view! {
                    <div class="card-grid">
                        {list.into_iter().map(|book| view! { <BookCard book=book /> }).collect::<Vec<_>>()}
                    </div>
                }.into_any(),
            }}
        </div>
    }
}

#[component]
fn BookCard(book: BookWithAuthor) -> impl IntoView {
    let stars = render_stars(book.book.rating);
    let href = format!("/books/{}", book.book.id);

    view! {
        <A href=href attr:class="card book-card">
            <div class="book-cover">
                <svg viewBox="0 0 60 80" xmlns="http://www.w3.org/2000/svg">
                    <rect width="60" height="80" rx="2" fill="#e5e7eb"/>
                    <text x="30" y="44" text-anchor="middle" fill="#9ca3af" font-size="10">"Book"</text>
                </svg>
            </div>
            <div class="book-info">
                <h3 class="book-title">{book.book.title.clone()}</h3>
                <p class="book-author">{book.author_name.clone()}</p>
                <div class="book-meta">
                    {book.book.genre.as_ref().map(|g| view! { <span class="badge">{g.clone()}</span> })}
                    {book.book.published_year.map(|y| view! { <span class="text-muted book-year">{y.to_string()}</span> })}
                </div>
                <div class="stars">{stars}</div>
            </div>
        </A>
    }
}

fn render_stars(rating: Option<i32>) -> String {
    match rating {
        Some(r) => {
            let filled = (r.clamp(0, 5)) as usize;
            let empty = 5 - filled;
            "\u{2605}".repeat(filled) + &"\u{2606}".repeat(empty)
        }
        None => "\u{2606}\u{2606}\u{2606}\u{2606}\u{2606}".to_string(),
    }
}
