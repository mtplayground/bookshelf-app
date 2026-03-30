use leptos::prelude::*;
use leptos_router::components::A;

#[component]
pub fn BookForm() -> impl IntoView {
    view! {
        <div class="page-header">
            <A href="/" attr:class="btn">"Back to Books"</A>
        </div>
        <h1>"Add a Book"</h1>
        <p class="text-muted">"Book form will appear here."</p>
    }
}
