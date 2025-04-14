use crate::swap::Swap;
use leptos::prelude::{component, view};
use leptos_router::{components::Route, MatchNestedRoutes};
use leptos_router_macro::path;

#[component]
pub fn SwapRoutes() -> impl MatchNestedRoutes + Clone {
    view! {
        <Route path=path!("/liquidity-book-leptos/trade") view=Swap/>
    }
    .into_inner()
}
