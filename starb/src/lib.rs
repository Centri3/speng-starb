#![feature(decl_macro)]
#![feature(pointer_byte_offsets)]
#![feature(vec_into_raw_parts)]

pub mod app;
pub mod plugin;
mod plugins;
pub mod utils;

use app::StarApp;
use color_eyre::config::HookBuilder;
use eframe::NativeOptions;
use std::env::current_exe;
use std::fs::File;
use std::panic::set_hook;
use std::sync::Arc;
use std::thread;
use std::time::Duration;
use tracing::error;
use tracing::info;
use tracing::Level;
use windows_sys::Win32::Foundation::HMODULE;
use windows_sys::Win32::System::SystemServices::DLL_PROCESS_ATTACH;

// WARNING: ACTUALLY OK CODE BELOW

#[no_mangle]
unsafe extern "system" fn DllMain(_: HMODULE, reason: u32, _: usize) -> bool {
    if reason == DLL_PROCESS_ATTACH {
        thread::spawn(|| unsafe { __start_starb() });
    }

    true
}

unsafe fn __start_starb() {
    // Just incase SE somehow changed current_dir before we got here
    let log = current_exe()
        .expect("This can't be seen. No point")
        .with_file_name("starb.log");

    // Do not start if STARB_DONOTSTART exists, because why not?
    if log
        .with_file_name("STARB_DONOTSTART")
        .try_exists()
        .expect("This can't be seen. No point")
    {
        return;
    }

    tracing_subscriber::fmt()
        .with_writer(Arc::new(
            File::create(log).expect("This can't be seen. No point"),
        ))
        .with_max_level(Level::DEBUG)
        .with_ansi(true)
        .init();

    let (ph, eh) = HookBuilder::default()
        .display_env_section(false)
        .panic_section("Please report this at: https://github.com/Centri3/speng-starb/issues/new")
        .into_hooks();

    eh.install().expect("This can't be seen. No point");

    set_hook(Box::new(move |pi| {
        error!(
            "unexpected panic, handing off to color-eyre:\n\n{}",
            ph.panic_report(pi)
        );
    }));

    info!("Hii!! uwu");

    // FIXME: Not sure why this fixes SE getting stuck at "Initializing OpenGL"
    thread::sleep(Duration::from_secs(1u64));

    eframe::run_native(
        &format!("Star Browser Utilities v{}", env!("CARGO_PKG_VERSION")),
        NativeOptions::default(),
        Box::new(|cc| Box::new(StarApp::new(cc))),
    )
    .expect("Failed to start eframe");
}
