use leptos::prelude::*;
use leptos_router::components::A;

#[component]
pub fn Nav() -> impl IntoView {
    view! {
        <nav class="leading-tight flex flex-row items-center">
            <A
                exact=true
                strict_trailing_slash=false
                href="/liquidity-book-leptos/trade"
                attr:class="text-muted-foreground px-3 py-1.5 no-underline leading-none"
            >
                "Trade"
            </A>
            <A
                href="/liquidity-book-leptos/pool"
                attr:class="text-muted-foreground px-3 py-1.5 no-underline leading-none"
            >
                "Pool"
            </A>
            <a
                href="https://kent-3.github.io/liquidity-book/docs/"
                target="_blank"
                rel="noopener"
                class="text-muted-foreground px-3 py-1.5 no-underline leading-none"
            >
                "Docs"
            </a>
        </nav>
    }
}
