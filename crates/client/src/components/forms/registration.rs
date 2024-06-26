use leptos::{
    component, create_signal, event_target_value, view, Action, IntoView, Signal, SignalGet,
    SignalUpdate, SignalWith, WriteSignal,
};
use tracing::debug;
use wasm_bindgen::JsCast;

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

    fn on_change<T: JsCast>(ev: T, signal: WriteSignal<String>) {
        let val = event_target_value(&ev);
        signal.update(|v| *v = val)
    }

    view! {
        <form on:submit=|ev| ev.prevent_default()>
            {move || error.with(|e| e.as_ref().map(|e| view! {
                <p style="color:red">{e}</p>
            }))}

            <input
                type="text"
                required
                placeholder="Username"
                prop:autocomplete="username"
                prop:disabled=move || disabled.get()
                on:keyup=move |ev| on_change(ev, set_name)
                on:change=move |ev| on_change(ev, set_name)
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
