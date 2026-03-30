use leptos::prelude::*;
use leptos_router::components::A;
use types::AuthorSummary;

use crate::api;

#[component]
pub fn Authors() -> impl IntoView {
    let authors = RwSignal::new(None::<Result<Vec<AuthorSummary>, String>>);

    Effect::new(move |_| {
        leptos::task::spawn_local(async move {
            authors.set(Some(api::fetch_authors().await));
        });
    });

    view! {
        <div class="page-header">
            <h1>"Authors"</h1>
            <A href="/add-author" attr:class="btn btn-primary">"+ Add Author"</A>
        </div>
        {move || match authors.get() {
            None => view! { <div class="loading">"Loading authors..."</div> }.into_any(),
            Some(Err(e)) => view! { <div class="error-msg">{e}</div> }.into_any(),
            Some(Ok(ref list)) if list.is_empty() => view! {
                <div class="empty-state">
                    <p>"No authors yet."</p>
                    <A href="/add-author" attr:class="btn btn-primary">"Add your first author"</A>
                </div>
            }.into_any(),
            Some(Ok(list)) => view! {
                <div class="card-grid">
                    {list.into_iter().map(|a| view! { <AuthorCard author=a /> }).collect::<Vec<_>>()}
                </div>
            }.into_any(),
        }}
    }
}

#[component]
fn AuthorCard(author: AuthorSummary) -> impl IntoView {
    let href = format!("/authors/{}", author.id);
    let bio_preview = author
        .bio
        .as_deref()
        .map(|b| {
            if b.len() > 120 {
                format!("{}...", &b[..120])
            } else {
                b.to_string()
            }
        })
        .unwrap_or_default();
    let book_label = if author.book_count == 1 {
        "1 book".to_string()
    } else {
        format!("{} books", author.book_count)
    };

    view! {
        <A href=href attr:class="card author-card">
            <div class="author-avatar">
                {author.name.chars().next().unwrap_or('?').to_uppercase().to_string()}
            </div>
            <div class="author-info">
                <h3 class="author-name">{author.name.clone()}</h3>
                {if !bio_preview.is_empty() {
                    Some(view! { <p class="author-bio">{bio_preview}</p> })
                } else {
                    None
                }}
                <span class="badge">{book_label}</span>
            </div>
        </A>
    }
}
