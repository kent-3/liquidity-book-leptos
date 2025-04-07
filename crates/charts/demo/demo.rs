use ammber_charts::{load_data, LiquidityChart, PoolDistributionChart};
use leptos::prelude::*;

#[component]
fn App() -> impl IntoView {
    let debug = RwSignal::new(false);
    let data = Signal::stored(load_data());
    let token_labels = Signal::stored(("Token X".to_string(), "Token Y".to_string()));

    view! {
        <div>
            <h1>"Ammber Charts Demo"</h1>

            <div class="chart-card">
                <div class="chart-header">
                    <h2>"My Liquidity"</h2>
                </div>
                <div class="chart-container">
                    <LiquidityChart debug=debug.into() data=data token_labels=token_labels />
                </div>
            </div>

            <div class="chart-card">
                <div class="chart-header">
                    <h2>"Pool Distribution"</h2>
                </div>
                <div class="chart-container">
                    <PoolDistributionChart debug=debug.into() data=data token_labels=token_labels />
                </div>
            </div>

            <div class="controls">
                <label>
                    <input
                        type="checkbox"
                        prop:checked=move || debug.get()
                        on:change=move |_| debug.update(|v| *v = !*v)
                    />
                    "Show Debug Info"
                </label>
            </div>

        </div>
    }
}

fn main() {
    mount_to_body(|| view! { <App /> });
}
