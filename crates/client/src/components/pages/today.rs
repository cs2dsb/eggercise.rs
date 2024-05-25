use leptos::{component, view, IntoView, ReadSignal};

#[component]
pub fn Today(
    #[allow(unused_variables)]
    online: ReadSignal<bool>,
) -> impl IntoView {
    view! {
        <p>"Today"</p>
    }
}
