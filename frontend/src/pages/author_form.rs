use leptos::prelude::*;
use leptos_router::components::A;
use leptos_router::hooks::{use_navigate, use_params_map};
use types::{CreateAuthor, UpdateAuthor};

use crate::api;
use crate::toast::use_toast;

#[component]
pub fn AuthorForm() -> impl IntoView {
    let params = use_params_map();
    let nav = use_navigate();

    let edit_id = Memo::new(move |_| {
        params
            .read()
            .get("id")
            .and_then(|s| s.parse::<i64>().ok())
    });
    let is_edit = move || edit_id.get().is_some();

    let name = RwSignal::new(String::new());
    let bio = RwSignal::new(String::new());
    let error = RwSignal::new(None::<String>);
    let submitting = RwSignal::new(false);
    let loaded = RwSignal::new(false);

    // If editing, load existing author data
    Effect::new(move |_| {
        if let Some(id) = edit_id.get() {
            leptos::task::spawn_local(async move {
                match api::fetch_author(id).await {
                    Ok(data) => {
                        name.set(data.author.name);
                        bio.set(data.author.bio.unwrap_or_default());
                        loaded.set(true);
                    }
                    Err(e) => error.set(Some(e)),
                }
            });
        } else {
            loaded.set(true);
        }
    });

    let toast = use_toast();
    let on_submit = {
        let nav = nav.clone();
        move |ev: leptos::ev::SubmitEvent| {
            ev.prevent_default();
            error.set(None);

            let n = name.get();
            if n.trim().is_empty() {
                error.set(Some("Author name is required".to_string()));
                return;
            }

            let b = bio.get();
            let nav = nav.clone();
            submitting.set(true);

            if let Some(id) = edit_id.get() {
                let author = UpdateAuthor {
                    name: Some(n),
                    bio: Some(b),
                };
                leptos::task::spawn_local(async move {
                    match api::update_author(id, &author).await {
                        Ok(a) => {
                            toast.success("Author updated successfully");
                            nav(&format!("/authors/{}", a.id), Default::default());
                        }
                        Err(e) => {
                            submitting.set(false);
                            toast.error(&e);
                            error.set(Some(e));
                        }
                    }
                });
            } else {
                let author = CreateAuthor {
                    name: n,
                    bio: if b.is_empty() { None } else { Some(b) },
                };
                leptos::task::spawn_local(async move {
                    match api::create_author(&author).await {
                        Ok(a) => {
                            toast.success("Author created successfully");
                            nav(&format!("/authors/{}", a.id), Default::default());
                        }
                        Err(e) => {
                            submitting.set(false);
                            toast.error(&e);
                            error.set(Some(e));
                        }
                    }
                });
            }
        }
    };

    view! {
        <div class="page-header">
            <A href="/authors" attr:class="btn">"← Back to Authors"</A>
        </div>
        <h1>{move || if is_edit() { "Edit Author" } else { "Add an Author" }}</h1>

        {move || error.get().map(|e| view! { <div class="error-msg">{e}</div> })}

        <Show when=move || loaded.get() fallback=|| view! { <div class="loading">"Loading..."</div> }>
            <form class="card" on:submit=on_submit.clone()>
                <div class="form-group">
                    <label>"Name *"</label>
                    <input
                        type="text"
                        placeholder="Author name"
                        prop:value=move || name.get()
                        on:input=move |ev| name.set(event_target_value(&ev))
                        required
                    />
                </div>

                <div class="form-group">
                    <label>"Biography"</label>
                    <textarea
                        placeholder="Brief biography..."
                        prop:value=move || bio.get()
                        on:input=move |ev| bio.set(event_target_value(&ev))
                        rows="5"
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
                                "Update Author"
                            } else {
                                "Add Author"
                            }
                        }}
                    </button>
                    <A href="/authors" attr:class="btn">"Cancel"</A>
                </div>
            </form>
        </Show>
    }
}
