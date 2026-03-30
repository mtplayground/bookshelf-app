use leptos::prelude::*;
use leptos_router::components::{A, Route, Router, Routes};
use leptos_router::path;

use crate::pages::author_detail::AuthorDetail;
use crate::pages::author_form::AuthorForm;
use crate::pages::authors::Authors;
use crate::pages::book_detail::BookDetail;
use crate::pages::book_form::BookForm;
use crate::pages::home::Home;
use crate::pages::not_found::NotFound;

#[component]
pub fn App() -> impl IntoView {
    view! {
        <Router>
            <header>
                <div class="header-inner">
                    <A href="/" attr:class="logo">"Bookshelf"</A>
                    <nav>
                        <A href="/" attr:class="nav-link">"Books"</A>
                        <A href="/authors" attr:class="nav-link">"Authors"</A>
                        <A href="/stats" attr:class="nav-link">"Stats"</A>
                    </nav>
                </div>
            </header>
            <main>
                <Routes fallback=NotFound>
                    <Route path=path!("/") view=Home />
                    <Route path=path!("/books/:id") view=BookDetail />
                    <Route path=path!("/add-book") view=BookForm />
                    <Route path=path!("/edit-book/:id") view=BookForm />
                    <Route path=path!("/authors") view=Authors />
                    <Route path=path!("/authors/:id") view=AuthorDetail />
                    <Route path=path!("/add-author") view=AuthorForm />
                </Routes>
            </main>
        </Router>
    }
}
