use std::fmt::Display;

use leptos::leptos_dom::helpers::location as leptos_loc;

use crate::api::error::{FrontendError, ResultContext};

/// Gets the site host with no protocol or path 
pub fn host<T: Display>() -> Result<String, FrontendError<T>> {
    let loc = leptos_loc();
    let host = loc
        .host()
        // TODO: can we get rid of this manual map by implementing ResultContext/ErrorContext
        // for Into<FEE>
        .map_err(FrontendError::from)
        .context("location.host")?;

    Ok(host)
}

/// Gets the protocol (http/https) including a trailing ':'
pub fn protocol<T: Display>() -> Result<String, FrontendError<T>> {
    let loc = leptos_loc();
    let protocol = loc
        .protocol()
        // TODO: can we get rid of this manual map by implementing ResultContext/ErrorContext
        // for Into<FEE>
        .map_err(FrontendError::from)
        .context("location.protocol")?;

    Ok(protocol)
}

/// Gets the fully qualified URL to the site root including protocol
pub fn root_url<T: Display>() -> Result<String, FrontendError<T>> {
    let protocol = protocol()?;
    let host = host()?;

    Ok(format!("{protocol}//{host}"))
}