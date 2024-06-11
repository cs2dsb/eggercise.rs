use leptos::{
    component, create_local_resource, provide_context, view, CollectView, ErrorBoundary, IntoView,
    SignalWith, Transition,
};
use leptos_router::Router;

use crate::{
    components::{Container, Footer},
    db::migrations::{self, MigrationError},
    utils::sqlite3::SqlitePromiser,
    AppNav, AppRoutes,
};

#[component]
pub fn App() -> impl IntoView {
    let dbsetup = create_local_resource(
        || (),
        |_| async {
            let promiser = SqlitePromiser::use_promiser();

            promiser.configure().await?;
            let db_version = migrations::run_migrations(&promiser).await?;
            Ok::<_, MigrationError>(db_version)
        },
    );

    provide_context(dbsetup);

    view! {
        <Router>
            <AppNav/>
            <Container>
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
                        <AppRoutes/>
                    </ErrorBoundary>
                </Transition>
            </Container>
            <Footer/>
        </Router>
    }
}
