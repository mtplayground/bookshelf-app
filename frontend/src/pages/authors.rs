use leptos::prelude::*;
use leptos_router::components::A;

#[component]
pub fn Authors() -> impl IntoView {
    view! {
        <div class="page-header">
            <h1>"Authors"</h1>
            <A href="/add-author" attr:class="btn btn-primary">"+ Add Author"</A>
        </div>
        <p class="text-muted">"Your authors will appear here."</p>
    }
}
