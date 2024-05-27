use leptos::{
    component, create_action, create_signal,
    view, IntoView, Show, Signal, SignalGet, SignalUpdate, SignalWith,
    ReadSignal,
};
use shared::model::LoginUser;
use tracing::{debug, warn};

use crate::{api::login, components::{LoginForm, OfflineFallback}, ClientRoutes};

#[component]
pub fn Login(
    online: ReadSignal<bool>,
) -> impl IntoView {
    // Signals
    let (login_response, set_login_response) = create_signal(None);
    let (login_error, set_login_error) = create_signal(None::<String>);
    let (wait_for_response, set_wait_for_response) = create_signal(false);
    let disabled = Signal::derive(move || wait_for_response.get());

    // Actions
    let login_action = create_action(move |username: &String| {
        let login_user = LoginUser::new(username);
        debug!("Logging in user {:?}", login_user);
        async move {
            set_wait_for_response.update(|w| *w = true);

            match login(&login_user).await {
                Ok(res) => {
                    set_login_response.update(|v| *v = Some(res));
                    set_login_error.update(|e| *e = None);
                }
                Err(err) => {
                    let msg = format!("{:?}", err);
                    warn!("Error logging in {login_user:?}: {msg}");
                    set_login_error.update(|e| *e = Some(msg));
                }
            }

            set_wait_for_response.update(|w| *w = false);
        }
    });

    view! {
        <h2>"Login"</h2>
        <OfflineFallback online>
            <Show
                when=move || login_response.with(|r| r.is_some())
                fallback=move || {
                    view! {
                        <LoginForm
                            action=login_action
                            error=login_error
                            disabled
                        />
                    }
                }
            >
                <p>"Login complete"</p>
                { ClientRoutes::Today.link() }
                <div>{ login_response.with(|v| format!("{:?}", v)) }</div>
            </Show>
        </OfflineFallback>
    }
}
