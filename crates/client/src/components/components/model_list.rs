use std::{any::type_name, marker::PhantomData};

use leptos::{component, view, CollectView, IntoView, Transition};

use crate::{
    components::FrontendErrorBoundary, db::PromiserFetcher, utils::sqlite3::SqlitePromiserError,
};

#[component]
pub fn ModelList<T>(#[prop(optional)] _phantom: PhantomData<T>) -> impl IntoView
where
    T: PromiserFetcher + 'static,
{
    let values = T::all_resource();

    view! {
        <h3>{ type_name::<T>() }:</h3>
        <Transition fallback=move || view! {  <p>"Loading..."</p>} >
            <FrontendErrorBoundary<SqlitePromiserError>>
                { move || {
                    values.and_then(|l| l.into_view()).collect_view()
                }}
            </FrontendErrorBoundary<SqlitePromiserError>>
        </Transition>
    }
}
