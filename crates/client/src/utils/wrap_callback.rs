use wasm_bindgen::{closure::Closure, convert::FromWasmAbi, JsCast};
use web_sys::js_sys::Function;

/// Wraps a function in a JS closure and drops forgets the rust object
/// associated with it This leaks the memory associated with it so should only
/// be used for closures expected to live for the lifetime of the program. See
/// the documentations [js_sys::Closure::into_js_value] for details
pub fn wrap_callback<F, S, T>(s: S, mut f: F)
where
    F: FnMut(T) + 'static,
    S: FnOnce(&Function),
    T: FromWasmAbi + 'static,
{
    let callback = Closure::wrap(Box::new(move |t: T| f(t)) as Box<dyn FnMut(_)>);

    s(callback.as_ref().unchecked_ref());

    // Prevent it from being dropped
    callback.forget()
}
