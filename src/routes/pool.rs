mod pool_browser;
mod pool_creator;
mod pool_manager;

pub use pool_browser::PoolBrowser;
pub use pool_creator::PoolCreator;
pub use pool_manager::{AddLiquidity, PoolManager, RemoveLiquidity};

use leptos::prelude::*;
use leptos_router::nested_router::Outlet;
use tracing::info;

#[component]
pub fn Pool() -> impl IntoView {
    info!("rendering <Pool/>");

    on_cleanup(move || {
        info!("cleaning up <Pool/>");
    });

    // Resources in this component can be shared with all children, so they only run once and are
    // persistent. This is just an example:
    // let resource = LocalResource::new(move || {
    //     SendWrapper::new(async move {
    //         QueryMsg::GetNumberOfLbPairs {}
    //             .do_query(&LB_FACTORY)
    //             .await
    //     })
    // });

    // provide_context(resource);

    view! { <Outlet /> }
}

// NOTE: If the Router gets complicated enough, it's possible to split it up like this:

// use leptos_router::{
//     components::{ParentRoute, Route},
//     MatchNestedRoutes,
// };
// use leptos_router_macro::path;
//
// #[component]
// pub fn PoolRoutes() -> impl MatchNestedRoutes<Dom> + Clone {
//     view! {
//         <ParentRoute path=path!("/pool") view=Pool>
//             <Route path=path!("/") view=PoolBrowser />
//             <Route path=path!("/create") view=PoolCreator />
//             <ParentRoute path=path!("/:token_a/:token_b/:bps") view=PoolManager>
//                 <Route path=path!("/") view=|| () />
//                 <Route path=path!("/add") view=AddLiquidity />
//                 <Route path=path!("/remove") view=RemoveLiquidity />
//             </ParentRoute>
//         </ParentRoute>
//     }
//     .into_inner()
// }
