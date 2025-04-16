use ammber_core::{Endpoint, KeplrSignals, TokenMap};
use ammber_swap::Swap;
use leptos::prelude::*;
use tracing::{debug, info};

#[component]
pub fn Trade() -> impl IntoView {
    info!("rendering <Trade/>");

    on_cleanup(move || {
        info!("cleaning up <Trade/>");
    });

    view! { <Swap /> }
}
