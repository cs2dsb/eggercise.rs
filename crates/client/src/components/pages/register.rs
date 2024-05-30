use leptos::{
    component, create_action, create_signal, view, IntoView, Show, Signal, SignalGet, SignalUpdate,
    SignalWith,
};
use shared::model::RegistrationUser;
use tracing::{debug, warn};

use crate::{
    api::register,
    components::{OfflineFallback, RegistrationForm},
    ClientRoutes,
};

#[component]
pub fn Register() -> impl IntoView {
    // Signals
    let (register_response, set_register_response) = create_signal(None);
    let (register_error, set_register_error) = create_signal(None::<String>);
    let (wait_for_response, set_wait_for_response) = create_signal(false);
    let disabled = Signal::derive(move || wait_for_response.get());

    // Actions
    let register_action = create_action(move |username: &String| {
        let reg_user = RegistrationUser::new(username);
        debug!("Registering new user {:?}", reg_user);
        async move {
            set_wait_for_response.update(|w| *w = true);

            match register(&reg_user).await {
                Ok(res) => {
                    set_register_response.update(|v| *v = Some(res));
                    set_register_error.update(|e| *e = None);
                }
                Err(err) => {
                    let msg = format!("{:?}", err);
                    warn!("Error registering {reg_user:?}: {msg}");
                    set_register_error.update(|e| *e = Some(msg));
                }
            }

            set_wait_for_response.update(|w| *w = false);
        }
    });

    view! {
        <h2>"Register"</h2>
        <OfflineFallback>
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
                <p>"Registration complete"</p>
                <span>"You can now "</span> { ClientRoutes::Login.link() }
            </Show>
        </OfflineFallback>
    }
}
