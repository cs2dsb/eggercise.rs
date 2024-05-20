use leptos::{component, view, IntoView};
use leptos_router::{Route, Routes, A};

use crate::components::{Plan, Today, Register, Login, Profile};

macro_rules! routes {
    ($(($path:literal, $view:ident, $ui_text:literal),)+) => {
        #[component]
        pub fn AppNav() -> impl IntoView {
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

routes!(
    ("/", Today, "Today"), 
    ("/plan", Plan, "Plan"),
    ("/register", Register, "Register"),
    ("/login", Login, "Login"),
    ("/profile", Profile, "Profile"),
);
