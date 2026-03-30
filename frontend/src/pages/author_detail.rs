use leptos::prelude::*;
use leptos_router::components::A;
use leptos_router::hooks::{use_navigate, use_params_map};
use types::AuthorWithBooks;

use crate::api;

#[component]
pub fn AuthorDetail() -> impl IntoView {
    let params = use_params_map();
    let nav = use_navigate();
    let author = RwSignal::new(None::<Result<AuthorWithBooks, String>>);
    let deleting = RwSignal::new(false);

    Effect::new(move |_| {
        let id_str = params.read().get("id").unwrap_or_default();
        let id: i64 = id_str.parse().unwrap_or(0);
        if id > 0 {
            leptos::task::spawn_local(async move {
                author.set(Some(api::fetch_author(id).await));
            });
        } else {
            author.set(Some(Err("Invalid author ID".to_string())));
        }
    });

    view! {
        <div class="page-header">
            <A href="/authors" attr:class="btn">"← Back to Authors"</A>
        </div>
        {move || match author.get() {
            None => view! { <div class="loading">"Loading author..."</div> }.into_any(),
            Some(Err(e)) => view! { <div class="error-msg">{e}</div> }.into_any(),
            Some(Ok(data)) => {
                let edit_href = format!("/edit-author/{}", data.author.id);
                let delete_id = data.author.id;
                let nav = nav.clone();
                let book_count = data.books.len();
                view! {
                    <div class="detail-header">
                        <h1>{data.author.name.clone()}</h1>
                        <div class="detail-meta">
                            <span>{format!("{} book{}", book_count, if book_count == 1 { "" } else { "s" })}</span>
                            <span>{format!("Added {}", data.author.created_at)}</span>
                        </div>
                    </div>

                    {data.author.bio.as_ref().map(|bio| view! {
                        <div class="card" style="margin-bottom: 1.5rem">
                            <p>{bio.clone()}</p>
                        </div>
                    })}

                    <div class="detail-actions">
                        <A href=edit_href attr:class="btn btn-primary">"Edit Author"</A>
                        <button
                            class="btn btn-danger"
                            on:click=move |_| {
                                let nav = nav.clone();
                                deleting.set(true);
                                leptos::task::spawn_local(async move {
                                    match api::delete_author(delete_id).await {
                                        Ok(()) => nav("/authors", Default::default()),
                                        Err(e) => {
                                            deleting.set(false);
                                            author.set(Some(Err(e)));
                                        }
                                    }
                                });
                            }
                            disabled=move || deleting.get()
                        >
                            {move || if deleting.get() { "Deleting..." } else { "Delete Author" }}
                        </button>
                    </div>

                    <h2 style="margin-top: 2rem; margin-bottom: 1rem">"Books"</h2>
                    {if data.books.is_empty() {
                        view! { <p class="text-muted">"This author has no books yet."</p> }.into_any()
                    } else {
                        view! {
                            <div class="card-grid">
                                {data.books.into_iter().map(|book| {
                                    let href = format!("/books/{}", book.id);
                                    let stars = render_stars(book.rating);
                                    view! {
                                        <A href=href attr:class="card book-card">
                                            <div class="book-info">
                                                <h3 class="book-title">{book.title.clone()}</h3>
                                                <div class="book-meta">
                                                    {book.genre.as_ref().map(|g| view! { <span class="badge">{g.clone()}</span> })}
                                                    {book.published_year.map(|y| view! { <span class="text-muted book-year">{y.to_string()}</span> })}
                                                </div>
                                                <div class="stars">{stars}</div>
                                            </div>
                                        </A>
                                    }
                                }).collect::<Vec<_>>()}
                            </div>
                        }.into_any()
                    }}
                }.into_any()
            }
        }}
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
