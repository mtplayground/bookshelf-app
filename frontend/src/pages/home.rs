use leptos::prelude::*;
use leptos_router::components::A;

#[component]
pub fn Home() -> impl IntoView {
    view! {
        <div class="page-header">
            <h1>"Books"</h1>
            <A href="/add-book" attr:class="btn btn-primary">"+ Add Book"</A>
        </div>
        <p class="text-muted">"Your book collection will appear here."</p>
    }
}
