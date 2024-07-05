pub use gloo_net::http::Method;
pub use stdext::function_name as stdext_function_name;

#[macro_export]
macro_rules! function_name {
    () => {{
        let name: &str = $crate::utils::server_log::stdext_function_name!();
        name.trim_end_matches("::{{closure}}")
    }};
}

#[macro_export]
macro_rules! server_log {
    ($level:literal, $message:expr, { $($json:tt)* }) => {
        let mut message = $message.map_or(String::new(), |v| v.into());

        let fn_name = $crate::function_name!();
        let sep = if message.len() > 0 { ": " } else { "" };

        // limit the number of expansions and not overallocate hopefully
        message.reserve_exact(fn_name.len() + sep.len());
        message.insert_str(0, fn_name);
        message.insert_str(fn_name.len(), sep);

        let payload = serde_json::json!({
            "level": $level,
            "message": message,
            $($json)*
        });

        if let Err(e) = $crate::utils::fetch::json_request::<_,(), $crate::api::error::Nothing>(
            $crate::utils::server_log::Method::POST,
            $crate::api::Object::Log.path(),
            Some(&$crate::api::error::NoValidation(&payload))).await
        {
            web_sys::console::error_1(&wasm_bindgen::JsValue::from_str(&format!("Error sending log to server:\n{e}\nPayload:\n{:?}", payload)));
        }
    };
    ($level:literal $(,)?) => {
        $crate::server_log!($level, None::<String>, {});
    };
    ($level:literal, { $($json:tt)* }) => {
        $crate::server_log!($level, None::<String>, { $($json)* }) ;
    };
    ($level:literal, $message:expr) => {
        $crate::server_log!($level, Some($message), {}) ;
    };
    ($level:literal, $message:expr, ( $($args:tt)+ )) => {
        let message = format_args!($message, $($args)+).to_string();
        $crate::server_log!($level, Some(message), {}) ;
    };
    ($level:literal, $message:expr, ( $($args:tt)+ ), { $($json:tt)* }) => {
        let message = format_args!($message, $($args)+).to_string();
        $crate::server_log!($level, Some(message), { $($json)+ }) ;
    };
}

#[macro_export]
macro_rules! server_trace {
    ({ $($tt:tt)* }) => { $crate::server_log!("trace", None::<String>, { $($tt)* }) };
    ($($tt:tt)*) => { $crate::server_log!("trace", $($tt)*) };
}

#[macro_export]
macro_rules! server_debug {
    ({ $($tt:tt)* }) => { $crate::server_log!("debug", None::<String>, { $($tt)* }) };
    ($($tt:tt)*) => { $crate::server_log!("debug", $($tt)*) };
}

#[macro_export]
macro_rules! server_info {
    ({ $($tt:tt)* }) => { $crate::server_log!("info", None::<String>, { $($tt)* }) };
    ($($tt:tt)*) => { $crate::server_log!("info", $($tt)*) };
}

#[macro_export]
macro_rules! server_warn {
    ({ $($tt:tt)* }) => { $crate::server_log!("warn", None::<String>, { $($tt)* }) };
    ($($tt:tt)*) => { $crate::server_log!("warn", $($tt)*) };
}

#[macro_export]
macro_rules! server_error {
    ({ $($tt:tt)* }) => { $crate::server_log!("error", None::<String>, { $($tt)* }) };
    ($($tt:tt)*) => { $crate::server_log!("error", $($tt)*) };
}

#[allow(unused)]
async fn turds() {
    server_log!("trace", "Hello");
    server_log!("trace", "Hello {}", ("world"));
    server_log!("trace", "Hello {}", ("world"), { "hello": "world" });
    server_log!("trace", Some("Hello"), { "hello": "world" });

    server_trace!("Hello");
    server_trace!("Hello {}", ("world"));
    server_trace!("Hello {}", ("world"), { "hello": "world" });
    server_trace!(Some("Hello"), { "hello": "world" });
    server_trace!({ "hello": "world" });

    server_debug!("Hello");
    server_debug!("Hello {}", ("world"));
    server_debug!("Hello {}", ("world"), { "hello": "world" });
    server_debug!(Some("Hello"), { "hello": "world" });

    server_info!("Hello");
    server_info!("Hello {}", ("world"));
    server_info!("Hello {}", ("world"), { "hello": "world" });
    server_info!(Some("Hello"), { "hello": "world" });

    server_warn!("Hello");
    server_warn!("Hello {}", ("world"));
    server_warn!("Hello {}", ("world"), { "hello": "world" });
    server_warn!(Some("Hello"), { "hello": "world" });

    server_error!("Hello");
    server_error!("Hello {}", ("world"));
    server_error!("Hello {}", ("world"), { "hello": "world" });
    server_error!(Some("Hello"), { "hello": "world" });
}
