use leptos::{component, view, IntoView};

use crate::components::OnlineCheck;

#[component]
pub fn Footer() -> impl IntoView {
    view! {
        <ul class="footer">
            <li>
                <OnlineCheck />
            </li>
            <li>
                <small>{
                    format!("Version: {}{}",
                        env!("CARGO_PKG_VERSION"),
                        option_env!("BUILD_TIME")
                            .map(|v| format!(" - {v}"))
                            .unwrap_or("".to_string()))
                }</small>
            </li>
        </ul>
    }
}
