use leptos::prelude::*;
use leptos_router::components::A;

#[component]
pub fn Nav() -> impl IntoView {
    view! {
        <nav class="px-2 sm:px-4 leading-tight flex flex-row items-center">
            <A exact=true strict_trailing_slash=false href="/liquidity-book-leptos/">
                "Trade"
            </A>
            <A href="/liquidity-book-leptos/pool">"Pool"</A>
            <a href="https://kent-3.github.io/liquidity-book/docs/" target="_blank" rel="noopener">
                "Docs"
            </a>
        </nav>
    }
}
