use leptos::{component, view, IntoView, ReadSignal};

#[component]
pub fn Plan(
    #[allow(unused_variables)]
    online: ReadSignal<bool>,
) -> impl IntoView {
    view! {
        <p>"Plan"</p>
    }
}
