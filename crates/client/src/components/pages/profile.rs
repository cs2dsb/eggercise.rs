use leptos::{
    component, create_action, create_local_resource, create_signal, view, Action, IntoView, Show,
    Signal, SignalGet, SignalUpdate, SignalWith,
};
use shared::{
    api::error::{Nothing, ServerError},
    model::{TemporaryLogin, User},
};
use tracing::{debug, warn};

use crate::{
    api::{add_key, create_temporary_login, fetch_user},
    components::{
        forms::CreateTemporaryLoginForm, AddKeyForm, FrontendErrorBoundary, OfflineFallback,
    },
};

type ServerErrorNothing = ServerError<Nothing>;

#[component]
fn ProfileWithUser(
    user: Signal<Option<User>>,
    temporary_login: Signal<Option<TemporaryLogin>>,
    update_action: Action<(User, TemporaryLogin), ()>,
) -> impl IntoView {
    let (temporary_login_error, set_temporary_login_error) = create_signal(None::<String>);
    let (add_key_error, set_add_key_error) = create_signal(None::<String>);
    let (add_key_success, set_add_key_success) = create_signal(None::<String>);
    let (wait_for_response, set_wait_for_response) = create_signal(false);

    let create_temporary_login_action = create_action(move |_: &()| {
        debug!("Creating temporary login...");
        async move {
            set_wait_for_response.update(|w| *w = true);

            match create_temporary_login().await {
                Ok(tl) => {
                    set_temporary_login_error.update(|e| *e = None);
                    update_action.dispatch((user.get().unwrap(), tl))
                }
                Err(err) => {
                    let msg = format!("{:?}", err);
                    warn!("Error creating temporary login: {msg}");
                    set_temporary_login_error.update(|e| *e = Some(msg));
                }
            }

            set_wait_for_response.update(|w| *w = false);
        }
    });

    let add_key_action = create_action(move |_: &()| {
        debug!("Adding key...");
        async move {
            set_wait_for_response.update(|w| *w = true);

            match add_key().await {
                Ok(_) => {
                    set_add_key_success.update(|v| *v = Some("Key added successfully".to_string()));
                    set_add_key_error.update(|e| *e = None);
                }
                Err(err) => {
                    let msg = format!("{:?}", err);
                    warn!("Error adding key: {msg}");
                    set_add_key_success.update(|v| *v = None);
                    set_add_key_error.update(|e| *e = Some(msg));
                }
            }

            set_wait_for_response.update(|w| *w = false);
        }
    });

    view! {
        <Show
            when=move || user.with(|u| u.is_some())
            fallback=move || {
                view! {
                    <p>Loading...</p>
                }
            }
        >
            <h3>"You are logged in"</h3>
            <div><span>"Username: "</span><span>{ user.with(move |u| u.as_ref().map(|u| u.username.clone() )) }</span></div>
            <Show
                when=move || temporary_login.with(|tl| tl.is_some())
                fallback=move || {
                    view! {
                        <CreateTemporaryLoginForm
                            action=create_temporary_login_action
                            error=temporary_login_error
                            disabled=wait_for_response
                        />
                    }
                }
            >
                {move || temporary_login.with(move |tl| tl.as_ref().map(|tl| {
                    view! {
                        <a href={ &tl.url }>{ &tl.url }</a>
                        <img src={ tl.qr_code_url() }/>
                    }
                }))}
            </Show>

            <AddKeyForm
                action=add_key_action
                error=add_key_error
                message=add_key_success
                disabled=wait_for_response
            />
        </Show>
    }
}

#[component]
pub fn Profile() -> impl IntoView {
    // Resources
    let user_and_temp_login = create_local_resource(move || (), |_| fetch_user());

    let update_action = create_action(move |(user, tl): &(User, TemporaryLogin)| {
        let user = user.clone();
        let tl = tl.clone();

        async move {
            user_and_temp_login.update(|v| {
                *v = Some(Ok((user, Some(tl))));
            })
        }
    });

    let user = Signal::derive(move || match user_and_temp_login.get() {
        Some(Ok((u, _))) => Some(u),
        _ => None,
    });

    let temporary_login = Signal::derive(move || match user_and_temp_login.get() {
        Some(Ok((_, tl))) => tl,
        _ => None,
    });

    view! {
        <h2>"Profile"</h2>
        <OfflineFallback>
            <div>
                <FrontendErrorBoundary<ServerErrorNothing>>
                { move || user_and_temp_login.and_then(|_| {
                    view! {
                        <ProfileWithUser
                            user
                            temporary_login
                            update_action
                        />
                    }
                })}
                </FrontendErrorBoundary<ServerErrorNothing>>
            </div>
        </OfflineFallback>
    }
}
