use crate::{ChildrenFn, Show};
use core::time::Duration;
use leptos::component;
use leptos::prelude::*;
// use leptos_reactive::{
//     create_render_effect, on_cleanup, signal_prelude::*, store_value, StoredValue,
// };
use tracing::debug;

/// A component that will show its children when the `when` condition is `true`.
/// Additionally, you need to specify a `hide_delay`. If the `when` condition changes to `false`,
/// the unmounting of the children will be delayed by the specified Duration.
/// If you provide the optional `show_class` and `hide_class`, you can create very easy mount /
/// unmount animations.
///
/// ```rust
/// # use core::time::Duration;
/// # use leptos::*;
/// # #[component]
/// # pub fn App() -> impl IntoView {
/// let show = create_rw_signal(false);
///
/// view! {
///     <div
///         class="hover-me"
///         on:mouseenter=move |_| show.set(true)
///         on:mouseleave=move |_| show.set(false)
///     >
///         "Hover Me"
///     </div>
///
///     <AnimatedShow
///        when=show
///        show_class="fadeIn"
///        hide_class="fadeOut"
///        hide_delay=Duration::from_millis(1000)
///     >
///        <div class="here-i-am">
///            "Here I Am!"
///        </div>
///     </AnimatedShow>
/// }
/// # }
/// ```
#[component]
pub fn AnimatedShow(
    /// The components Show wraps
    children: ChildrenFn,
    /// If the component should show or not
    #[prop(into)]
    when: MaybeSignal<bool>,
    /// Optional CSS class to apply if `when == true`
    #[prop(optional)]
    show_class: &'static str,
    /// Optional CSS class to apply if `when == false`
    #[prop(optional)]
    hide_class: &'static str,
    /// The timeout after which the component will be unmounted if `when == false`
    hide_delay: Duration,
) -> impl IntoView {
    let handle: StoredValue<Option<TimeoutHandle>> = StoredValue::new(None);
    let cls = RwSignal::new(if when.get_untracked() {
        show_class
    } else {
        hide_class
    });
    let show = RwSignal::new(when.get_untracked());

    Effect::new(move |_| {
        if when.get() {
            // clear any possibly active timer
            if let Some(h) = handle.get_value() {
                h.clear();
            }

            debug!("true, adding show_class");
            cls.set(show_class);
            show.set(true);
        } else {
            cls.set(hide_class);
            debug!("false, adding hide_class");

            let h = set_timeout_with_handle(move || show.set(false), hide_delay)
                .expect("set timeout in AnimatedShow");
            handle.set_value(Some(h));
        }
    });

    on_cleanup(move || {
        if let Some(Some(h)) = handle.try_get_value() {
            h.clear();
        }
    });

    view! {
        <Show when=move || show.get() fallback=|| ()>
            <div class=move || cls.get()>{children()}</div>
        </Show>
    }
}
