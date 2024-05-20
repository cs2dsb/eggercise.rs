use leptos::{component, create_local_resource, view, IntoView, SignalWith, ErrorBoundary, CollectView};
use shared::model::User;
use crate::api::fetch_user;

#[component]
pub fn UserView(user: User) -> impl IntoView {
    view! {
        <p>{ user.username } </p>
    }
}

#[component]
pub fn Profile() -> impl IntoView {

    let user = create_local_resource(move || (), |_| fetch_user());

    view! {
        <div>
            <p>"With EB"</p>
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
                }
            })}
            </ErrorBoundary>
            
        </div>
    }
    /*
    // Resources
    let user = create_local_resource(|| (), |_| async move {
        fetch_user().await
    });

    let cats_view = move || {
        user.and_then(|data| {
            view! { <p> { format!("{:?}", data) } </p> }
        })
    };

    view! {
        <h2>"Profile"</h2>
        <ErrorBoundary fallback=|errors| view! {
            <div style="color:red">
                <p>Error:</p>
                <ul>
                { move || errors.with(|v| 
                    v.iter()
                     .map(|(_, e)| view! { <li> { e.to_string() } </li>})
                     .collect_view())
                }
                </ul>
            </div>
        }>
            <Show 
                when=move || user.loading().get()
                fallback=move || {
                    view! {
                        <p>"Fetching user..."</p>
                    }
                }
            > 
                <p>"Current user:"</p>
                <div>{ cats_view }</div>
            </Show>
        </ErrorBoundary>
    }
    */
}
