use leptos::prelude::*;
use leptos_router::components::A;
use types::BookStats;

use crate::api;

#[component]
pub fn Stats() -> impl IntoView {
    let stats = RwSignal::new(None::<Result<BookStats, String>>);

    Effect::new(move |_| {
        leptos::task::spawn_local(async move {
            stats.set(Some(api::fetch_book_stats().await));
        });
    });

    view! {
        <div class="page-header">
            <h1>"Statistics"</h1>
        </div>
        {move || match stats.get() {
            None => view! { <div class="loading">"Loading statistics..."</div> }.into_any(),
            Some(Err(e)) => view! { <div class="error-msg">{e}</div> }.into_any(),
            Some(Ok(data)) => view! {
                <div class="stat-grid">
                    <div class="stat-card">
                        <div class="stat-value">{data.total_books.to_string()}</div>
                        <div class="stat-label">"Total Books"</div>
                    </div>
                    <div class="stat-card">
                        <div class="stat-value">{data.total_authors.to_string()}</div>
                        <div class="stat-label">"Total Authors"</div>
                    </div>
                    <div class="stat-card">
                        <div class="stat-value">
                            {data.avg_rating.map(|r| format!("{:.1}", r)).unwrap_or_else(|| "—".to_string())}
                        </div>
                        <div class="stat-label">"Average Rating"</div>
                    </div>
                </div>

                <h2 style="margin-bottom: 1rem">"Books by Genre"</h2>
                {if data.books_by_genre.is_empty() {
                    view! { <p class="text-muted" style="margin-bottom: 1.5rem">"No genre data yet."</p> }.into_any()
                } else {
                    view! {
                        <div class="card" style="margin-bottom: 1.5rem">
                            {data.books_by_genre.into_iter().map(|g| {
                                view! {
                                    <div style="display: flex; justify-content: space-between; padding: 0.5rem 0; border-bottom: 1px solid var(--border);">
                                        <span class="badge">{g.genre}</span>
                                        <span class="text-muted">{format!("{} book{}", g.count, if g.count == 1 { "" } else { "s" })}</span>
                                    </div>
                                }
                            }).collect::<Vec<_>>()}
                        </div>
                    }.into_any()
                }}

                <h2 style="margin-bottom: 1rem">"Top Rated Books"</h2>
                {if data.top_rated.is_empty() {
                    view! { <p class="text-muted" style="margin-bottom: 1.5rem">"No rated books yet."</p> }.into_any()
                } else {
                    view! {
                        <div class="card-grid" style="margin-bottom: 1.5rem">
                            {data.top_rated.into_iter().map(|book| {
                                let href = format!("/books/{}", book.id);
                                let stars = render_stars(book.rating);
                                view! {
                                    <A href=href attr:class="card book-card">
                                        <div class="book-info">
                                            <h3 class="book-title">{book.title}</h3>
                                            <p class="book-author">{book.author_name}</p>
                                            <div class="stars">{stars}</div>
                                        </div>
                                    </A>
                                }
                            }).collect::<Vec<_>>()}
                        </div>
                    }.into_any()
                }}

                <h2 style="margin-bottom: 1rem">"Most Prolific Authors"</h2>
                {if data.most_prolific.is_empty() {
                    view! { <p class="text-muted" style="margin-bottom: 1.5rem">"No authors yet."</p> }.into_any()
                } else {
                    view! {
                        <div class="card-grid" style="margin-bottom: 1.5rem">
                            {data.most_prolific.into_iter().map(|author| {
                                let href = format!("/authors/{}", author.id);
                                let book_label = format!("{} book{}", author.book_count, if author.book_count == 1 { "" } else { "s" });
                                view! {
                                    <A href=href attr:class="card author-card">
                                        <div class="author-avatar">
                                            {author.name.chars().next().unwrap_or('?').to_uppercase().to_string()}
                                        </div>
                                        <div class="author-info">
                                            <h3 class="author-name">{author.name}</h3>
                                            <span class="badge">{book_label}</span>
                                        </div>
                                    </A>
                                }
                            }).collect::<Vec<_>>()}
                        </div>
                    }.into_any()
                }}

                <h2 style="margin-bottom: 1rem">"Recently Added"</h2>
                {if data.recently_added.is_empty() {
                    view! { <p class="text-muted">"No books added yet."</p> }.into_any()
                } else {
                    view! {
                        <div class="card-grid">
                            {data.recently_added.into_iter().map(|book| {
                                let href = format!("/books/{}", book.id);
                                view! {
                                    <A href=href attr:class="card book-card">
                                        <div class="book-info">
                                            <h3 class="book-title">{book.title}</h3>
                                            <p class="book-author">{book.author_name}</p>
                                            <p class="text-muted" style="font-size: 0.8rem">{format!("Added {}", book.created_at)}</p>
                                        </div>
                                    </A>
                                }
                            }).collect::<Vec<_>>()}
                        </div>
                    }.into_any()
                }}
            }.into_any(),
        }}
    }
}

fn render_stars(rating: i32) -> String {
    let filled = (rating.clamp(0, 5)) as usize;
    let empty = 5 - filled;
    "\u{2605}".repeat(filled) + &"\u{2606}".repeat(empty)
}
