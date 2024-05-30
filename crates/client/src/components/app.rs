use leptos::{component, create_local_resource, view, IntoView, Transition, ErrorBoundary, SignalWith, CollectView};
use leptos_router::Router;

use crate::{components::OnlineCheck, db::migrations::{self, MigrationError}, utils::sqlite3::SqlitePromiser, AppNav, AppRoutes};

#[component]
pub fn App() -> impl IntoView {
    let dbsetup = create_local_resource(|| (), |_| async { 
        let promiser = SqlitePromiser::use_promiser();

        promiser.configure().await?;
        let db_version = migrations::run_migrations(&promiser).await?;
        let opfs_tree = promiser.opfs_tree().await?;
        Ok::<_, MigrationError>((opfs_tree, db_version))
    });

    view! {
        <Router>
            <Transition fallback=move || view! {  <p>"Loading..."</p>} >
                <ErrorBoundary fallback=|errors| view! {
                    <div style="color:red">
                        <p>Error configuring and migrating database:</p>
                        <ul>
                        { move || errors.with(|v|
                            v.iter()
                            .map(|(_, e)| view! { <li> { format!("{:?}", e) } </li>})
                            .collect_view())
                        }
                        </ul>
                    </div>
                }>
                    <OnlineCheck />
                    <p><small>{ 
                        format!("Version: {}{}", 
                            env!("CARGO_PKG_VERSION"),
                            option_env!("BUILD_TIME")
                                .map(|v| format!(" - {v}"))
                                .unwrap_or("".to_string())) 
                    }</small></p>
                    <p><small>"DB Version: "{ dbsetup.and_then(|v| v.1) }</small></p>
                    <p><small>"Opfs tree: "{ dbsetup.and_then(|v| format!("{:#?}", v.0)) }</small></p>
                    
                    <AppNav/>
                    <AppRoutes/>
                </ErrorBoundary>
            </Transition>
        </Router>
    }
}
