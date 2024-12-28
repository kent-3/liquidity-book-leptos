use leptos::prelude::*;
use leptos_router::components::A;

#[component]
pub fn Nav() -> impl IntoView {
    view! {
        <header>
            <nav>
                <A href="/liquidity-book-leptos/">"Home"</A>
                <A href="/liquidity-book-leptos/pool">"Pool"</A>
                <A href="/liquidity-book-leptos/trade">"Trade"</A>
                <A href="/liquidity-book-leptos/analytics">"Analytics"</A>
            </nav>
        </header>
    }
}
