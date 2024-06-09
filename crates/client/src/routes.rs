use leptos::{component, view, IntoView};
use leptos_router::{Route, Routes, A};

use crate::components::{Login, Plan, Profile, Register, Today, Debug, OnlineCheck};

macro_rules! routes {
    ($(($path:literal, $view:ident, $ui_text:literal),)+) => {
        #[component]
        pub fn AppNav() -> impl IntoView {
            view! {
                <ul class="nav full-width">
                $(
                    <li>
                        <A href=$path>$ui_text</A>
                    </li>
                )+
                    <li id="right">
                        <small>{
                            format!("Version: {}{}",
                                env!("CARGO_PKG_VERSION"),
                                option_env!("BUILD_TIME")
                                    .map(|v| format!(" - {v}"))
                                    .unwrap_or("".to_string()))
                        }</small>
                    </li>
                    <li>
                        <OnlineCheck />
                    </li>
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
);
