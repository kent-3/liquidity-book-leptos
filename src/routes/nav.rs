use leptos::prelude::*;
use leptos_router::components::A;

#[component]
pub fn Nav() -> impl IntoView {
    view! {
        <nav>
            <A exact=true href="/liquidity-book-leptos/">
                "Home"
            </A>
            <A href="/liquidity-book-leptos/trade">"Trade"</A>
            <A href="/liquidity-book-leptos/pool">"Pool"</A>
        </nav>
    }
}
