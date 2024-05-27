use leptos::{
    component, create_signal, ev::KeyboardEvent, event_target_value, view, Action, IntoView,
    Signal, SignalGet, SignalUpdate, SignalWith,
};
use tracing::debug;

#[component]
pub fn RegistrationForm(
    action: Action<String, ()>,
    #[prop(into)] error: Signal<Option<String>>,
    disabled: Signal<bool>,
) -> impl IntoView {
    let (name, set_name) = create_signal(String::new());
    let dispatch_action = move || {
        let n = name.get();
        debug!("RegistrationForm::dispatch_action: {n}");
        action.dispatch(n)
    };
    let button_disabled = Signal::derive(move || disabled.get() || name.with(|n| n.is_empty()));

    view! {
        <form on:submit=|ev| ev.prevent_default()>
            {move || error.with(|e| e.as_ref().map(|e| view! {
                <p style="color:red">{e}</p>
            }))}

            <input
                type="text"
                required
                placeholder="Username"
                prop:disabled=move || disabled.get()
                // TODO: Is it possible to dedupe these?
                on:keyup=move |ev: KeyboardEvent| {
                    let val = event_target_value(&ev);
                    set_name.update(|v| *v = val);
                }
                on:change=move |ev| {
                    let val = event_target_value(&ev);
                    set_name.update(|v| *v = val);
                }
            />

            <button
                prop:disabled=move || button_disabled.get()
                on:click=move |_| dispatch_action()
            >
                "Register"
            </button>

        </form>
    }
}
