// Please don't judge me for my spaghetti code here please I'm rushing this out really hard and this project shouldn't be taken too seriously anyway (it'll be made obsolete soon™️)

#![cfg_attr(
    all(target_os = "windows", not(debug_assertions)),
    windows_subsystem = "windows"
)]

use app::StarApp;
use eframe::NativeOptions;

mod app;
mod handler;

fn main() {
    // This line cost me an hour of debugging. Yep, an hour. I had just forgotten to remove this line...
    // handler::EsiFilter::new(&handler::Handler::new());

    eframe::run_native(
        format!("{}-{}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION")).as_str(),
        NativeOptions::default(),
        Box::new(|_| Box::new(StarApp::new())),
    );
}
