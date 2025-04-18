use crate::pool::{
    AddLiquidity, Pool, PoolAnalytics, PoolBrowser, PoolCreator, PoolManager, Pools,
    RemoveLiquidity,
};
use leptos::prelude::{component, view};
use leptos_router::{
    components::{ParentRoute, Redirect, Route},
    MatchNestedRoutes,
};
use leptos_router_macro::path;

#[component]
pub fn PoolRoutes() -> impl MatchNestedRoutes + Clone {
    view! {
        <ParentRoute path=path!("/liquidity-book-leptos/pool") view=Pools>
            <Route path=path!("") view=PoolBrowser />
            <Route path=path!("create") view=PoolCreator />
            <ParentRoute path=path!("/:token_a/:token_b/:bps") view=Pool>
                <Route path=path!("") view=|| view! { <Redirect path="manage" /> } />
                <ParentRoute path=path!("/manage") view=PoolManager>
                    <Route path=path!("") view=AddLiquidity />
                    <Route path=path!("add") view=AddLiquidity />
                    <Route path=path!("remove") view=RemoveLiquidity />
                </ParentRoute>
                <Route path=path!("analytics") view=PoolAnalytics />
            </ParentRoute>
        </ParentRoute>
    }
    .into_inner()
}
