use leptos::prelude::*;
use leptos_router::components::A;

#[component]
pub fn NotFound() -> impl IntoView {
    view! {
        <div class="not-found">
            <h1>"404"</h1>
            <p>"The page you're looking for doesn't exist."</p>
            <A href="/" attr:class="btn btn-primary">"Go Home"</A>
        </div>
    }
}
