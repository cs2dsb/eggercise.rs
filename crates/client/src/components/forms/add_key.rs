use leptos::{component, view, Action, IntoView, Signal, SignalGet, SignalWith};

#[component]
pub fn AddKeyForm(
    action: Action<(), ()>,
    #[prop(into)]
    error: Signal<Option<String>>,
    disabled: Signal<bool>,
) -> impl IntoView 
{
    let dispatch_action = move || action.dispatch(());
    let button_disabled = Signal::derive(move || disabled.get());
        
    view! {
        <form on:submit=|ev| ev.prevent_default()>
            {move || error.with(|e| e.as_ref().map(|e| view! {
                <p style="color:red">{e}</p>
            }))}

            <button
                prop:disabled=move || button_disabled.get()
                on:click=move |_| dispatch_action()
            >
                "Enroll new key"
            </button>
        </form>
    }
}
