use leptos::{component, view, IntoView};
use leptos_router::{Route, Routes, A};

use super::{Plan, Today};

macro_rules! routes {
    ($(($path:literal, $view:ident, $ui_text:literal),)+) => {
        #[component]
        pub (crate) fn AppNav() -> impl IntoView {
            view! {
                <nav>
                    <ul>
                    $(
                        <li>
                            <A href=$path>
                                $ui_text
                            </A>
                            // TODO: remove once CCS is done
                            <span style:margin="10px" />
                        </li>
                    )+
                    </ul>
                </nav>
            }
        }

        #[component(transparent)]
        pub (crate) fn AppRoutes() -> impl IntoView {
            view! {
                <Routes>
                $(
                    <Route path=$path view=$view/>
                )+
                </Routes>
            }
        }
    };
    ($path:literal, $view:ident, $ui_text:literal) => {
        routes!(($path, $view, $ui_text),);
    };
    (($path:literal, $view:ident, $ui_text:literal)) => {
        routes!(($path, $view, $ui_text),);
    };
}

routes!(("", Today, "Today"), ("plan", Plan, "Plan"),);
