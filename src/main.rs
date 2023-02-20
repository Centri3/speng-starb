#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

#[macro_use]
extern crate eyre;

#[macro_use]
extern crate tracing;

mod exe;
mod serde;

use color_eyre::config::HookBuilder;
use exe::EXE;
use eyre::Result;
use eyre::WrapErr;
use std::env;
use std::fs;
use std::io;
use std::panic;
use target_lexicon::HOST;
use tracing_appender::non_blocking::NonBlocking;
use tracing_appender::non_blocking::WorkerGuard;
use tracing_appender::rolling::never;
use tracing_error::ErrorLayer;
use tracing_subscriber::fmt;
use tracing_subscriber::fmt::writer::MakeWriterExt;
use tracing_subscriber::prelude::*;
use tracing_subscriber::registry;
use tracing_subscriber::EnvFilter;

// Linux users: <https://gist.github.com/michaelbutler/f364276f4030c5f449252f2c4d960bd2>
#[cfg(not(all(target_arch = "x86_64", target_os = "windows")))]
compile_error!("`Star Browser Utilities` should only be compiled on `Windows`");

fn main() {
    let _guard = __setup_logging().expect("Failed to setup logging");

    EXE.init("SpaceEngine.exe").unwrap();
    EXE.read_to::<u32>(1000).unwrap();
}

/// Extracted from `main()`
#[inline(always)]
#[instrument(level = "trace")]
fn __setup_logging() -> Result<WorkerGuard> {
    // Backtrace should only be enabled in debug mode
    #[cfg(debug_assertions)]
    env::set_var("RUST_BACKTRACE", "full");

    // We don't care if this fails, as it means the log didn't exist already
    fs::remove_file("starb.log").ok();

    let (log_file, guard) = tracing_appender::non_blocking(never("", "starb.log"));

    __setup_tracing(log_file)?;

    __setup_hooks()?;

    trace!("Logging successfully setup");

    __print_debug_info()?;

    // Return guard to guarantee everything is logged before closing
    Ok(guard)
}

/// Extracted from `__setup_logging()`
#[inline(always)]
#[instrument(level = "trace")]
fn __setup_tracing(log_file: NonBlocking) -> Result<()> {
    // We want logs in release mode to be a little less verbose
    #[cfg(debug_assertions)]
    const ENV_FILTER: &str = "trace";
    #[cfg(not(debug_assertions))]
    const ENV_FILTER: &str = "debug";

    let fmt_layer = fmt::layer()
        .with_writer(log_file.and(io::stdout))
        .with_thread_names(true);

    registry()
        .with(ErrorLayer::default())
        .with(EnvFilter::try_new(ENV_FILTER)?)
        .with(fmt_layer)
        .init();

    Ok(())
}

/// Extracted from `__setup_logging()`
#[inline(always)]
#[instrument(level = "trace")]
fn __setup_hooks() -> Result<()> {
    // Setup color-eyre with custom settings
    let (ph, eh) = HookBuilder::default()
        .display_env_section(false)
        .panic_section("Please report this at: https://github.com/Centri3/speng-starb/issues/new")
        .into_hooks();

    eh.install().wrap_err("Failed to install color-eyre")?;

    panic::set_hook(Box::new(move |pi| {
        error!(
            "Panicked, handing off to color-eyre:\n\n{}",
            ph.panic_report(pi),
        );
    }));

    Ok(())
}

/// Extracted from `__setup_logging()`
#[inline(always)]
#[instrument(level = "trace")]
fn __print_debug_info() -> Result<()> {
    debug!(starb_version = env!("CARGO_PKG_VERSION"));
    debug!(target_triple = %HOST);

    Ok(())
}
