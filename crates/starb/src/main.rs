#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
mod patches;

use starb_handler::Handler;
use sysinfo::{System, SystemExt};

#[cfg(not(target_os = "windows"))]
compile_error!("speng-starb is windows only, sorry.");

fn main() {
    let mut sys = System::new_all();
    sys.refresh_all();

    let handler = Handler::new(&mut sys);

    println!(
        "{:x?}",
        handler.read(handler.base() + 0x69ad70, 4usize)
    );

    // eframe::run_native(
    //     format!(
    //         "Star Browser Utilities ({} {})",
    //         env!("CARGO_PKG_NAME"),
    //         env!("CARGO_PKG_VERSION"),
    //     )
    //     .as_str(),
    //     NativeOptions {
    //         drag_and_drop_support: false,
    //         ..Default::default()
    //     },
    //     Box::new(|cc| Box::new(StarbApp::new(cc))),
    // );
}
