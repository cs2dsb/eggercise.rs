use std::{error::Error, fmt::Display, marker::PhantomData};

use leptos::{
    component, create_memo, create_rw_signal,
    leptos_dom::{Errors, HydrationCtx, IntoView},
    provide_context, run_as_child, view, Children, CollectView, RwSignal, SignalGet, SignalWith,
};
use shared::api::error::FrontendError;

/// Copied from <https://github.com/leptos-rs/leptos/blob/8606f3d928899ebff237d541c51028df780402e3/leptos/src/error_boundary.rs>
/// This version has a hardcoded fallback
#[component]
pub fn FrontendErrorBoundary<T>(
    /// The components inside the tag which will get rendered
    children: Children,
    #[prop(optional)] _ty: PhantomData<T>,
) -> impl IntoView
where
    T: Display + 'static,
    FrontendError<T>: Error,
{
    run_as_child(move || {
        let before_children = HydrationCtx::next_error();

        let errors: RwSignal<Errors> = create_rw_signal(Errors::default());

        provide_context(errors);

        // Run children so that they render and execute resources
        _ = HydrationCtx::next_error();
        let children = children();
        HydrationCtx::continue_from(before_children);

        // #[cfg(all(debug_assertions, feature = "hydrate"))]
        // {
        //     use leptos_dom::View;
        //     if children.nodes.iter().any(|child| {
        //         matches!(child, View::Suspense(_, _))
        //         || matches!(child, View::Component(repr) if repr.name() == "Transition")
        //     }) {
        //         leptos_dom::logging::console_warn("You are using a <Suspense/> or \
        //         <Transition/> as the direct child of an <ErrorBoundary/>. To ensure
        // correct \         hydration, these should be reorganized so that the
        // <ErrorBoundary/> is a child \         of the <Suspense/> or
        // <Transition/> instead: \n\         \nview! {{ \
        //         \n  <Suspense fallback=todo!()>\n    <ErrorBoundary
        // fallback=todo!()>\n      {{move || {{ /* etc. */")     }
        // }

        let children = children.into_view();
        let errors_empty = create_memo(move |_| errors.with(Errors::is_empty));

        move || {
            if errors_empty.get() {
                children.clone().into_view()
            } else {
                view! {
                    <ul class="error-list">
                    { move || errors
                        .get()
                        .into_iter()
                        .map(|(_, e)| {
                            let e = e.into_inner();
                            if let Some(e) = e.downcast_ref::<FrontendError<T>>() {
                                e.into_view()
                            } else {
                                // Fallback to just display
                                view! { <li> { format!("{}", e) } </li>}
                                    .into_view()
                            }
                        })
                        .collect_view()
                    }
                    </ul>
                    <leptos-error-boundary style="display: none">{children.clone()}</leptos-error-boundary>
                }
                .into_view()
            }
        }
    })
}
