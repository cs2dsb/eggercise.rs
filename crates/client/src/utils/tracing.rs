use time::format_description::well_known::Iso8601;
use tracing_web::{MakeWebConsoleWriter, performance_layer};
use tracing_subscriber::{
    fmt::{format::Pretty, time::UtcTime}, 
    layer::SubscriberExt, 
    util::SubscriberInitExt,
};

pub fn configure_tracing() {
    let fmt_layer = tracing_subscriber::fmt::layer()
        .with_ansi(false) // Only partially supported across browsers
        // .without_time()   // std::time is not available in browsers
        .with_timer(UtcTime::new(Iso8601::DEFAULT))
        .with_writer(MakeWebConsoleWriter::new()); // write events to the console
    let perf_layer = performance_layer()
        .with_details_from_fields(Pretty::default());

    tracing_subscriber::registry()
        .with(fmt_layer)
        .with(perf_layer)
        .init(); // Install these as subscribers to tracing events
}