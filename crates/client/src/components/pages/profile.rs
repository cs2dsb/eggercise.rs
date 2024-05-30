use leptos::{
    component, create_action, create_local_resource, create_signal, view, Action, CollectView,
    ErrorBoundary, IntoView, Show, Signal, SignalGet, SignalUpdate, SignalWith,
};
use shared::model::{TemporaryLogin, User};
use tracing::{debug, warn};

use crate::{
    api::{create_temporary_login, fetch_user},
    components::{forms::CreateTemporaryLoginForm, OfflineFallback},
};

#[component]
fn ProfileWithUser(
    user: Signal<Option<User>>,
    temporary_login: Signal<Option<TemporaryLogin>>,
    update_action: Action<(User, TemporaryLogin), ()>,
) -> impl IntoView {
    let (temporary_login_error, set_temporary_login_error) = create_signal(None::<String>);
    let (wait_for_response, set_wait_for_response) = create_signal(false);

    let create_temporary_login_action = create_action(move |_: &()| {
        debug!("Adding new key...");
        async move {
            set_wait_for_response.update(|w| *w = true);

            match create_temporary_login().await {
                Ok(tl) => {
                    set_temporary_login_error.update(|e| *e = None);
                    update_action.dispatch((user.get().unwrap(), tl))
                }
                Err(err) => {
                    let msg = format!("{:?}", err);
                    warn!("Error adding key: {msg}");
                    set_temporary_login_error.update(|e| *e = Some(msg));
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
                // <div><span>"Temp login: "</span><span>{ temporary_login.with(move |tl| tl.as_ref().map(|tl| tl.url.clone() )) }</span></div>
            </Show>
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
                <ErrorBoundary fallback=|errors| view! {
                    <div style="color:red">
                        <p>Error:</p>
                        <ul>
                        { move || errors.with(|v|
                            v.iter()
                            .map(|(_, e)| view! { <li> { format!("{:?}", e) } </li>})
                            .collect_view())
                        }
                        </ul>
                    </div>
                }>
                { move || user_and_temp_login.and_then(|_| {
                    view! {
                        <ProfileWithUser
                            user
                            temporary_login
                            update_action
                        />
                    }
                })}
                </ErrorBoundary>
            </div>
        </OfflineFallback>
    }
}
