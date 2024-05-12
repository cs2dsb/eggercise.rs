use leptos::{component, create_action, create_signal, logging::{log, warn}, view, IntoView, Show, Signal, SignalGet, SignalUpdate, SignalWith};
use shared::model::{NewUser, User};

use crate::{api::register, components::RegistrationForm};

#[component]
pub(crate) fn Register() -> impl IntoView {
    // Signals
    let (register_response, set_register_response) = create_signal(None::<User>);
    let (register_error, set_register_error) = create_signal(None::<String>);
    let (wait_for_response, set_wait_for_response) = create_signal(false);
    let disabled = Signal::derive(move || wait_for_response.get());

    // Actions
    let register_action = create_action(move |name: &String| {
        let new_user = NewUser::new(name);
        log!("Registering new user {:?}", new_user);
        async move {
            set_wait_for_response.update(|w| *w = true);
            
            match register(&new_user).await {
                Ok(res) => {
                    set_register_response.update(|v| *v = Some(res));
                    set_register_error.update(|e| *e = None);
                }, 
                Err(err) => {   
                    let msg = format!("{err}");
                    warn!("Error registering {new_user:?}: {msg}");
                    set_register_error.update(|e| *e = Some(msg));
                },
            }
            
            set_wait_for_response.update(|w| *w = false);
        }
    });

    view! {
        <h2>"Register"</h2>
        <Show 
            when=move || register_response.with(|r| r.is_some())
            fallback=move || {
                view! {
                    <RegistrationForm 
                        action=register_action
                        error=register_error
                        disabled
                    />
                }
            }
        >
            <p>"Registration successful!"</p>
            <p>{ format!("New user: {:#?}", register_response.get()) }</p>
        </Show>
    }
}
