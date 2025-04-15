use leptos::{ev, logging::*, prelude::*};
use web_sys::wasm_bindgen::JsCast;

// just an idea
#[component]
pub fn KeyboardShortcuts() -> impl IntoView {
    let handle_shortcut = move |ev: web_sys::KeyboardEvent| {
        let target = ev.target();

        // Check if the event is coming from an input field
        if let Some(target) = target.and_then(|t| t.dyn_into::<web_sys::HtmlElement>().ok()) {
            let tag = target.tag_name().to_lowercase();
            if tag == "input" || tag == "textarea" || target.is_content_editable() {
                return; // Don't trigger shortcut inside input fields
            }
        }
        if ev.ctrl_key() {
            // Check if Ctrl is held
            match ev.code().as_str() {
                "Digit1" => log!("Ctrl + 1 pressed → Action 1"),
                "Digit2" => log!("Ctrl + 2 pressed → Action 2"),
                "Digit3" => log!("Ctrl + 3 pressed → Action 3"),
                _ => {}
            }
        }
    };

    // Attach a global keydown listener
    window_event_listener(ev::keydown, handle_shortcut);

    view! {
        <p>
            "Keyboard shortcuts: Press "<kbd>"1"</kbd>", "<kbd>"2"</kbd>", or "<kbd>"3"</kbd>
            " to trigger actions."
        </p>
    }
}
