use leptos::{component, view, ChildrenFn, IntoView};

/// The top level app container. Everything except header/footer & nav live in
/// this div
#[component]
pub fn Container(children: ChildrenFn) -> impl IntoView {
    view! {
        <div class="container">
            {children()}
        </div>
    }
}
