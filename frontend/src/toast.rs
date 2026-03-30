use gloo_timers::callback::Timeout;
use leptos::prelude::*;

#[derive(Clone)]
pub struct Toast {
    pub message: String,
    pub kind: ToastKind,
}

#[derive(Clone, PartialEq)]
pub enum ToastKind {
    Success,
    Error,
}

#[derive(Clone, Copy)]
pub struct ToastState {
    toast: RwSignal<Option<Toast>>,
    counter: RwSignal<u32>,
}

impl ToastState {
    pub fn new() -> Self {
        Self {
            toast: RwSignal::new(None),
            counter: RwSignal::new(0),
        }
    }

    pub fn success(&self, message: impl Into<String>) {
        self.show(message.into(), ToastKind::Success);
    }

    pub fn error(&self, message: impl Into<String>) {
        self.show(message.into(), ToastKind::Error);
    }

    fn show(&self, message: String, kind: ToastKind) {
        let id = self.counter.get_untracked() + 1;
        self.counter.set(id);
        self.toast.set(Some(Toast { message, kind }));
        let toast = self.toast;
        let counter = self.counter;
        Timeout::new(3_000, move || {
            if counter.get_untracked() == id {
                toast.set(None);
            }
        })
        .forget();
    }
}

pub fn use_toast() -> ToastState {
    use_context::<ToastState>().expect("ToastState not provided")
}

#[component]
pub fn ToastContainer() -> impl IntoView {
    let state = use_context::<ToastState>().expect("ToastState not provided");

    view! {
        <div class="toast-container">
            {move || state.toast.get().map(|t| {
                let class = match t.kind {
                    ToastKind::Success => "toast toast-success",
                    ToastKind::Error => "toast toast-error",
                };
                let toast = state.toast;
                view! {
                    <div class=class>
                        <span>{t.message}</span>
                        <button class="toast-close" on:click=move |_| toast.set(None)>"\u{00d7}"</button>
                    </div>
                }
            })}
        </div>
    }
}
