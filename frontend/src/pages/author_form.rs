use leptos::prelude::*;
use leptos_router::components::A;

#[component]
pub fn AuthorForm() -> impl IntoView {
    view! {
        <div class="page-header">
            <A href="/authors" attr:class="btn">"Back to Authors"</A>
        </div>
        <h1>"Add an Author"</h1>
        <p class="text-muted">"Author form will appear here."</p>
    }
}
