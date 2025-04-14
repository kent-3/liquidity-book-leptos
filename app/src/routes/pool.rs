use leptos::prelude::*;
use leptos_router::nested_router::Outlet;
use tracing::info;

// this is currently being bypassed and using the Pools component from the ammber_pools crate

#[component]
pub fn Pools() -> impl IntoView {
    info!("rendering <Pools/>");

    on_cleanup(move || {
        info!("cleaning up <Pools/>");
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

    view! {
        <div class="pools-group">
            <Outlet />
        </div>
    }
}
