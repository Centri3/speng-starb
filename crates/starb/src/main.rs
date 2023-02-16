#[macro_use]
extern crate eyre;

#[macro_use]
extern crate tracing;

mod exe;

use crate::exe::EXE;
use color_eyre::config::HookBuilder;
use color_eyre::config::PanicHook;
use std::env;
use std::fs;
use std::io;
use std::panic;
use target_lexicon::HOST;
use tracing::subscriber::set_global_default;
use tracing::Level;
use tracing_appender::non_blocking::WorkerGuard;
use tracing_appender::rolling::never;
use tracing_subscriber::fmt;
use tracing_subscriber::fmt::writer::MakeWriterExt;

// Linux users: <https://gist.github.com/michaelbutler/f364276f4030c5f449252f2c4d960bd2>
#[cfg(not(all(target_arch = "x86_64", target_os = "windows")))]
compile_error!("`Star Browser Utilities` should only be compiled on `Windows`");

fn main() {
    let _guard = __setup_logging();

    EXE.init("SpaceEngine.exe").unwrap();
    EXE.read_to::<u32>(1).unwrap();
}

/// Extracted from `main()`
#[inline(always)]
#[instrument(level = "trace")]
fn __setup_logging() -> WorkerGuard {
    const LEVEL: Level = if cfg!(debug_assertions) {
        Level::TRACE
    } else {
        Level::INFO
    };

    // Backtrace should only be enabled in debug mode
    #[cfg(debug_assertions)]
    env::set_var("RUST_BACKTRACE", "full");

    // We don't care if this fails, as it means the log didn't exist already
    let _ = fs::remove_file("starb.log");

    let (log_file, guard) = tracing_appender::non_blocking(never("", "starb.log"));

    set_global_default(
        fmt()
            .with_writer(log_file.and(io::stdout))
            .with_thread_names(true)
            .with_max_level(LEVEL)
            .finish(),
    )
    .expect("Failed to setup tracing");

    // Setup color-eyre with custom settings
    let (ph, eh) = HookBuilder::default()
        .display_env_section(false)
        .panic_section("Please report this at: https://github.com/Centri3/speng-starb/issues/new")
        .into_hooks();

    eh.install().expect("Failed to install color-eyre");

    __setup_panic_hook(ph);

    trace!("Logging successfully setup");

    __print_debug_info();

    // Return guard to guarantee everything is logged before closing
    guard
}

/// Extracted from `__setup_logging()`
#[inline(always)]
#[instrument(skip(ph), level = "trace")]
fn __setup_panic_hook(ph: PanicHook) {
    panic::set_hook(Box::new(move |pi| {
        error!(
            "Panicked, handing off to color-eyre:\n\n{}",
            ph.panic_report(pi)
        );
    }));
}

/// Extracted from `__setup_logging()`
#[inline(always)]
#[instrument(level = "trace")]
fn __print_debug_info() {
    debug!(starb_version = env!("CARGO_PKG_VERSION"));
    debug!(target_triple = %HOST);
}
