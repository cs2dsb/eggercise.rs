use leptos::{component, view, IntoView};
use leptos_router::{Route, Routes, A};

use crate::components::{Chart, Debug, Login, Plan, Profile, Register, Today};

macro_rules! routes {
    ($(($path:literal, $view:ident, $ui_text:literal),)+) => {
        #[component]
        pub fn AppNav() -> impl IntoView {
            view! {
                <ul class="nav">
                $(
                    <li>
                        <A href=$path>$ui_text</A>
                    </li>
                )+
                </ul>
            }
        }

        #[derive(Debug, Clone, Copy)]
        pub enum ClientRoutes {
            $(
                $view,
            )+
        }

        impl ClientRoutes {
            #[allow(dead_code)]
            pub fn path(self) -> &'static str {
                match self {
                    $(
                        Self::$view => $path,
                    )+
                }
            }

            #[allow(dead_code)]
            pub fn ui_text(self) -> &'static str {
                match self {
                    $(
                        Self::$view => $ui_text,
                    )+
                }
            }

            #[allow(dead_code)]
            pub fn link(self) -> impl IntoView {
                match self {
                    $(
                        Self::$view => view! {
                            <A href=$path>$ui_text</A>
                        },
                    )+
                }
            }
        }

        #[component(transparent)]
        pub fn AppRoutes() -> impl IntoView {
            view! {
                <Routes>
                $(
                    <Route
                        path=$path
                        view=$view
                    />
                )+
                </Routes>
            }
        }

        pub const ROUTE_URLS: &[&str] = &[
            $(
                $path,
            )
            +
        ];
    };
    ($path:literal, $view:ident, $ui_text:literal) => {
        routes!(($path, $view, $ui_text),);
    };
    (($path:literal, $view:ident, $ui_text:literal)) => {
        routes!(($path, $view, $ui_text),);
    };
}

routes!(
    ("/", Today, "Today"),
    ("/plan", Plan, "Plan"),
    ("/register", Register, "Register"),
    ("/login", Login, "Login"),
    ("/profile", Profile, "Profile"),
    ("/debug", Debug, "Debug"),
    ("/chart", Chart, "Chart"),
);
