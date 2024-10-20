use leptos::prelude::*;
use leptos_router::components::A;

#[component]
pub fn Nav() -> impl IntoView {
    view! {
        <header>
            <nav>
                <A href="/trader-crow-leptos/">"Home"</A>
                <A href="/trader-crow-leptos/pool">"Pool"</A>
                <A href="/trader-crow-leptos/trade">"Trade"</A>
                <A href="/trader-crow-leptos/analytics">"Analytics"</A>
            </nav>
        </header>
    }
}
