//! Logging for Star Browser Utilities

#[macro_use]
extern crate tracing;

use color_eyre::config::HookBuilder;
use std::env;
use std::fs::File;
use std::io;
use std::panic;
use std::str::FromStr;
use tracing_error::ErrorLayer;
use tracing_subscriber::fmt;
use tracing_subscriber::prelude::*;
use tracing_subscriber::EnvFilter;

// Might as well keep this consistent everywhere too
pub const LOG_FILE: &str = "starb.log";

#[inline]
pub fn init() {
    // Backtrace should only be enabled in debug mode
    #[cfg(debug_assertions)]
    env::set_var("RUST_BACKTRACE", "full");

    // Create layers for tracing
    let env_filter = EnvFilter::from_str("trace").unwrap();
    let error = ErrorLayer::default();
    let fmt_stdout = fmt::layer().with_writer(io::stdout);
    let fmt_file =
        fmt::layer().with_writer(File::create(LOG_FILE).expect("Failed to create `LOG_FILE`"));

    tracing_subscriber::registry()
        .with(env_filter)
        .with(error)
        .with(fmt_stdout.with_thread_names(true))
        .with(fmt_file.with_thread_names(true))
        .init();

    let (ph, eh) = HookBuilder::default()
        .display_env_section(false)
        .panic_section("Please report this at: https://github.com/Centri3/speng-starb/issues/new")
        .into_hooks();

    eh.install().expect("Failed to install color-eyre");

    // Panics should be logged to a file while also being handled by color-eyre
    panic::set_hook(Box::new(move |pi| {
        error!("\n\n{}", ph.panic_report(pi));
    }));

    trace!("Logging successfully setup");
}
