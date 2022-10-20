#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

mod app;
mod handler;
mod patches;

use crate::app::StarbApp;
use eframe::NativeOptions;
use patches::Patch;
use sysinfo::{System, SystemExt};

#[derive(Debug, serde::Deserialize)]
struct Config {
    a: String,
    b: Patches,
}

#[derive(Debug, serde::Deserialize)]
struct Patches {
    b: String,
}

fn main() {
    let mut sys = System::new_all();
    sys.refresh_all();

    let x: crate::patches::compact::Compact = toml::from_slice(include_bytes!("patches/data/no_search_locking.toml")).unwrap();
    x.toggle();

    //
    //
    // let handler = crate::handler::Handler::new(&mut sys);
    //
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
