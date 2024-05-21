use leptos::{
    component, create_action, create_local_resource, create_signal, 
    logging::{log, warn}, view, CollectView, ErrorBoundary, IntoView, 
    Signal, SignalGet, SignalUpdate, SignalWith, Show,
};
use shared::model::User;
use crate::{
    api::{add_key, fetch_user},
    components::forms::AddKeyForm,
};

#[component]
pub fn UserView(user: User) -> impl IntoView {
    view! {
        <p>{ user.username } </p>
    }
}

#[component]
pub fn Profile() -> impl IntoView {
    // Resources
    let user = create_local_resource(move || (), |_| fetch_user());
    
    // Signals
    let (add_key_response, set_add_key_response) = create_signal(None);
    let (add_key_error, set_add_key_error) = create_signal(None::<String>);
    let (wait_for_response, set_wait_for_response) = create_signal(false);
    let disabled = Signal::derive(move || wait_for_response.get());

    // Actions
    let add_key_action = create_action(move |_: &()| {
        log!("Adding new key...");
        async move {
            set_wait_for_response.update(|w| *w = true);
            
            match add_key().await {
                Ok(res) => {
                    set_add_key_response.update(|v| *v = Some(res));
                    set_add_key_error.update(|e| *e = None);
                }, 
                Err(err) => {   
                    let msg = format!("{:?}", err);
                    warn!("Error adding key: {msg}");
                    set_add_key_error.update(|e| *e = Some(msg));
                },
            }
            
            set_wait_for_response.update(|w| *w = false);
        }
    });

    view! {
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
            { move || user.and_then(|u| {
                view! {
                    <UserView 
                        user=u.clone()
                    />
                    <Show 
                        when=move || add_key_response.with(|r| r.is_some())
                        fallback=move || {
                            view! {
                                <AddKeyForm
                                    action=add_key_action
                                    error=add_key_error
                                    disabled
                                />
                            }
                        }
                    > 
                        <p>"New key added"</p>
                    </Show>
                }
            })}
            </ErrorBoundary>
            
        </div>
    }
}
