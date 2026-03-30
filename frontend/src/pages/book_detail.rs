use leptos::prelude::*;
use leptos_router::hooks::use_params_map;
use leptos_router::components::A;

#[component]
pub fn BookDetail() -> impl IntoView {
    let params = use_params_map();
    let id = move || params.read().get("id").unwrap_or_default();

    view! {
        <div class="page-header">
            <A href="/" attr:class="btn">"Back to Books"</A>
        </div>
        <h1>{move || format!("Book #{}", id())}</h1>
        <p class="text-muted">"Book details will appear here."</p>
    }
}
