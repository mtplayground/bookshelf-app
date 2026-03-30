use leptos::prelude::*;
use leptos_router::components::A;
use leptos_router::hooks::{use_navigate, use_params_map};
use types::BookWithAuthor;

use crate::api;

#[component]
pub fn BookDetail() -> impl IntoView {
    let params = use_params_map();
    let nav = use_navigate();
    let book = RwSignal::new(None::<Result<BookWithAuthor, String>>);
    let deleting = RwSignal::new(false);

    Effect::new(move |_| {
        let id_str = params.read().get("id").unwrap_or_default();
        let id: i64 = id_str.parse().unwrap_or(0);
        if id > 0 {
            leptos::task::spawn_local(async move {
                book.set(Some(api::fetch_book(id).await));
            });
        } else {
            book.set(Some(Err("Invalid book ID".to_string())));
        }
    });

    view! {
        <div class="page-header">
            <A href="/" attr:class="btn">"← Back to Books"</A>
        </div>
        {move || match book.get() {
            None => view! { <div class="loading">"Loading book..."</div> }.into_any(),
            Some(Err(e)) => view! { <div class="error-msg">{e}</div> }.into_any(),
            Some(Ok(b)) => {
                let edit_href = format!("/edit-book/{}", b.book.id);
                let stars = render_stars(b.book.rating);
                let delete_id = b.book.id;
                let nav = nav.clone();
                view! {
                    <div class="detail-header">
                        <h1>{b.book.title.clone()}</h1>
                        <div class="detail-meta">
                            <span>"by " <strong>{b.author_name.clone()}</strong></span>
                            {b.book.genre.as_ref().map(|g| view! { <span class="badge">{g.clone()}</span> })}
                            {b.book.published_year.map(|y| view! { <span>{y.to_string()}</span> })}
                        </div>
                        <div class="stars">{stars}</div>
                    </div>
                    <div class="card">
                        {b.book.description.as_ref().map(|d| view! { <p style="margin-bottom: 1rem">{d.clone()}</p> })}
                        <div class="detail-fields">
                            {b.book.isbn.as_ref().map(|isbn| view! {
                                <div class="detail-field">
                                    <span class="detail-label">"ISBN"</span>
                                    <span>{isbn.clone()}</span>
                                </div>
                            })}
                            {b.book.cover_url.as_ref().map(|url| view! {
                                <div class="detail-field">
                                    <span class="detail-label">"Cover URL"</span>
                                    <span>{url.clone()}</span>
                                </div>
                            })}
                            <div class="detail-field">
                                <span class="detail-label">"Added"</span>
                                <span>{b.book.created_at.clone()}</span>
                            </div>
                        </div>
                    </div>
                    <div class="detail-actions">
                        <A href=edit_href attr:class="btn btn-primary">"Edit Book"</A>
                        <button
                            class="btn btn-danger"
                            on:click=move |_| {
                                let nav = nav.clone();
                                deleting.set(true);
                                leptos::task::spawn_local(async move {
                                    match api::delete_book(delete_id).await {
                                        Ok(()) => nav("/", Default::default()),
                                        Err(e) => {
                                            deleting.set(false);
                                            book.set(Some(Err(e)));
                                        }
                                    }
                                });
                            }
                            disabled=move || deleting.get()
                        >
                            {move || if deleting.get() { "Deleting..." } else { "Delete Book" }}
                        </button>
                    </div>
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
