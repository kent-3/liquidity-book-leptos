use leptos::prelude::*;
use leptos_router::components::A;

#[component]
pub fn Nav() -> impl IntoView {
    view! {
        <nav>
            <A href="/liquidity-book-leptos">"Trade"</A>
            <A href="/liquidity-book-leptos/pool">"Pool"</A>
            <a href="https://kent-3.github.io/liquidity-book/docs/" target="_blank" rel="noopener">
                "Docs"
            </a>
        </nav>
    }
}
