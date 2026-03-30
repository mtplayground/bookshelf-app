use leptos::prelude::*;
use leptos_router::components::A;
use leptos_router::hooks::{use_navigate, use_params_map};
use types::{Author, CreateBook, UpdateBook};

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
pub fn BookForm() -> impl IntoView {
    let params = use_params_map();
    let nav = use_navigate();

    let edit_id = Memo::new(move |_| {
        params
            .read()
            .get("id")
            .and_then(|s| s.parse::<i64>().ok())
    });
    let is_edit = move || edit_id.get().is_some();

    let title = RwSignal::new(String::new());
    let author_id = RwSignal::new(0_i64);
    let isbn = RwSignal::new(String::new());
    let published_year = RwSignal::new(String::new());
    let genre = RwSignal::new(String::new());
    let rating = RwSignal::new(0_i32);
    let cover_url = RwSignal::new(String::new());
    let description = RwSignal::new(String::new());

    let authors = RwSignal::new(None::<Result<Vec<Author>, String>>);
    let error = RwSignal::new(None::<String>);
    let submitting = RwSignal::new(false);
    let loaded = RwSignal::new(false);

    // Fetch authors for dropdown
    Effect::new(move |_| {
        leptos::task::spawn_local(async move {
            authors.set(Some(api::fetch_authors().await));
        });
    });

    // If editing, load existing book data
    Effect::new(move |_| {
        if let Some(id) = edit_id.get() {
            leptos::task::spawn_local(async move {
                match api::fetch_book(id).await {
                    Ok(b) => {
                        title.set(b.book.title);
                        author_id.set(b.book.author_id);
                        isbn.set(b.book.isbn.unwrap_or_default());
                        published_year.set(
                            b.book
                                .published_year
                                .map(|y| y.to_string())
                                .unwrap_or_default(),
                        );
                        genre.set(b.book.genre.unwrap_or_default());
                        rating.set(b.book.rating.unwrap_or(0));
                        cover_url.set(b.book.cover_url.unwrap_or_default());
                        description.set(b.book.description.unwrap_or_default());
                        loaded.set(true);
                    }
                    Err(e) => error.set(Some(e)),
                }
            });
        } else {
            loaded.set(true);
        }
    });

    let on_submit = {
        let nav = nav.clone();
        move |ev: leptos::ev::SubmitEvent| {
            ev.prevent_default();
            error.set(None);

            let t = title.get();
            if t.trim().is_empty() {
                error.set(Some("Title is required".to_string()));
                return;
            }
            let aid = author_id.get();
            if aid == 0 {
                error.set(Some("Please select an author".to_string()));
                return;
            }

            let r = rating.get();
            if r < 0 || r > 5 {
                error.set(Some("Rating must be between 0 and 5".to_string()));
                return;
            }

            let year_str = published_year.get();
            let year: Option<i32> = if year_str.trim().is_empty() {
                None
            } else {
                match year_str.trim().parse::<i32>() {
                    Ok(y) => Some(y),
                    Err(_) => {
                        error.set(Some("Invalid year".to_string()));
                        return;
                    }
                }
            };

            let isbn_val = isbn.get();
            let genre_val = genre.get();
            let cover_val = cover_url.get();
            let desc_val = description.get();

            let nav = nav.clone();
            submitting.set(true);

            if let Some(id) = edit_id.get() {
                let book = UpdateBook {
                    title: Some(t),
                    author_id: Some(aid),
                    isbn: Some(isbn_val),
                    published_year: year,
                    genre: Some(genre_val),
                    rating: Some(r),
                    cover_url: Some(cover_val),
                    description: Some(desc_val),
                };
                leptos::task::spawn_local(async move {
                    match api::update_book(id, &book).await {
                        Ok(b) => nav(&format!("/books/{}", b.id), Default::default()),
                        Err(e) => {
                            submitting.set(false);
                            error.set(Some(e));
                        }
                    }
                });
            } else {
                let book = CreateBook {
                    title: t,
                    author_id: aid,
                    isbn: if isbn_val.is_empty() {
                        None
                    } else {
                        Some(isbn_val)
                    },
                    published_year: year,
                    genre: if genre_val.is_empty() {
                        None
                    } else {
                        Some(genre_val)
                    },
                    rating: if r == 0 { None } else { Some(r) },
                    cover_url: if cover_val.is_empty() {
                        None
                    } else {
                        Some(cover_val)
                    },
                    description: if desc_val.is_empty() {
                        None
                    } else {
                        Some(desc_val)
                    },
                };
                leptos::task::spawn_local(async move {
                    match api::create_book(&book).await {
                        Ok(b) => nav(&format!("/books/{}", b.id), Default::default()),
                        Err(e) => {
                            submitting.set(false);
                            error.set(Some(e));
                        }
                    }
                });
            }
        }
    };

    view! {
        <div class="page-header">
            <A href="/" attr:class="btn">"← Back to Books"</A>
        </div>
        <h1>{move || if is_edit() { "Edit Book" } else { "Add a Book" }}</h1>

        {move || error.get().map(|e| view! { <div class="error-msg">{e}</div> })}

        <Show when=move || loaded.get() fallback=|| view! { <div class="loading">"Loading..."</div> }>
            <form class="card" on:submit=on_submit.clone()>
                <div class="form-group">
                    <label>"Title *"</label>
                    <input
                        type="text"
                        placeholder="Book title"
                        prop:value=move || title.get()
                        on:input=move |ev| title.set(event_target_value(&ev))
                        required
                    />
                </div>

                <div class="form-group">
                    <label>"Author *"</label>
                    {move || match authors.get() {
                        None => view! { <p class="text-muted">"Loading authors..."</p> }.into_any(),
                        Some(Err(e)) => view! { <p class="error-msg">{e}</p> }.into_any(),
                        Some(Ok(list)) => view! {
                            <select
                                prop:value=move || author_id.get().to_string()
                                on:change=move |ev| {
                                    let v: i64 = event_target_value(&ev).parse().unwrap_or(0);
                                    author_id.set(v);
                                }
                            >
                                <option value="0">"Select an author..."</option>
                                {list.into_iter().map(|a| {
                                    let id_str = a.id.to_string();
                                    view! { <option value=id_str>{a.name.clone()}</option> }
                                }).collect::<Vec<_>>()}
                            </select>
                        }.into_any(),
                    }}
                </div>

                <div class="form-row">
                    <div class="form-group">
                        <label>"ISBN"</label>
                        <input
                            type="text"
                            placeholder="ISBN-10 or ISBN-13"
                            prop:value=move || isbn.get()
                            on:input=move |ev| isbn.set(event_target_value(&ev))
                        />
                    </div>
                    <div class="form-group">
                        <label>"Published Year"</label>
                        <input
                            type="text"
                            placeholder="e.g. 2024"
                            prop:value=move || published_year.get()
                            on:input=move |ev| published_year.set(event_target_value(&ev))
                        />
                    </div>
                </div>

                <div class="form-group">
                    <label>"Genre"</label>
                    <select
                        prop:value=move || genre.get()
                        on:change=move |ev| genre.set(event_target_value(&ev))
                    >
                        <option value="">"Select genre..."</option>
                        {GENRES.iter().map(|g| view! {
                            <option value={*g}>{*g}</option>
                        }).collect::<Vec<_>>()}
                    </select>
                </div>

                <div class="form-group">
                    <label>"Rating"</label>
                    <StarInput rating=rating />
                </div>

                <div class="form-group">
                    <label>"Cover URL"</label>
                    <input
                        type="text"
                        placeholder="https://..."
                        prop:value=move || cover_url.get()
                        on:input=move |ev| cover_url.set(event_target_value(&ev))
                    />
                </div>

                <div class="form-group">
                    <label>"Description"</label>
                    <textarea
                        placeholder="Brief description of the book..."
                        prop:value=move || description.get()
                        on:input=move |ev| description.set(event_target_value(&ev))
                    ></textarea>
                </div>

                <div class="form-actions">
                    <button
                        type="submit"
                        class="btn btn-primary"
                        disabled=move || submitting.get()
                    >
                        {move || {
                            if submitting.get() {
                                "Saving..."
                            } else if is_edit() {
                                "Update Book"
                            } else {
                                "Add Book"
                            }
                        }}
                    </button>
                    <A href="/" attr:class="btn">"Cancel"</A>
                </div>
            </form>
        </Show>
    }
}

#[component]
fn StarInput(rating: RwSignal<i32>) -> impl IntoView {
    view! {
        <div class="star-input">
            {(1..=5).map(|n| {
                view! {
                    <button
                        type="button"
                        class="star-btn"
                        on:click=move |_| {
                            if rating.get() == n { rating.set(0) } else { rating.set(n) }
                        }
                    >
                        {move || if n <= rating.get() { "\u{2605}" } else { "\u{2606}" }}
                    </button>
                }
            }).collect::<Vec<_>>()}
            <span class="text-muted star-label">
                {move || {
                    let r = rating.get();
                    if r == 0 { "No rating".to_string() } else { format!("{r}/5") }
                }}
            </span>
        </div>
    }
}
