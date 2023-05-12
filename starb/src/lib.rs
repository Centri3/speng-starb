pub mod app;
pub mod plugin;

use app::MyEguiApp;
use color_eyre::config::HookBuilder;
use eframe::NativeOptions;
use std::env::current_exe;
use std::fs::File;
use std::panic::set_hook;
use std::sync::Arc;
use std::thread;
use std::time::Duration;
use steamworks_sys::SteamAPI_ISteamApps_GetAppBuildId;
use steamworks_sys::SteamAPI_Init;
use steamworks_sys::SteamAPI_RestartAppIfNecessary;
use steamworks_sys::SteamAPI_Shutdown;
use steamworks_sys::SteamAPI_SteamApps_v008;
use tracing::error;
use tracing::info;
use tracing::trace;
use tracing_subscriber::fmt::format::FmtSpan;
use windows_sys::w;
use windows_sys::Win32::Foundation::HMODULE;
use windows_sys::Win32::Foundation::HWND;
use windows_sys::Win32::System::SystemServices::DLL_PROCESS_ATTACH;
use windows_sys::Win32::UI::WindowsAndMessaging::MessageBoxW;
use windows_sys::Win32::UI::WindowsAndMessaging::MB_ICONERROR;
use windows_sys::Win32::UI::WindowsAndMessaging::MB_OK;

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
    let do_not_start = log
        .with_file_name("STARB_DONOTSTART")
        .try_exists()
        .expect("This can't be seen. No point");

    if do_not_start {
        return;
    }

    tracing_subscriber::fmt()
        .with_writer(Arc::new(
            File::create(log).expect("This can't be seen. No point"),
        ))
        .with_span_events(FmtSpan::FULL)
        .with_ansi(true)
        .init();

    let (ph, eh) = HookBuilder::default()
        .display_env_section(false)
        .panic_section("Please report this at: https://github.com/Centri3/speng-starb/issues/new")
        .into_hooks();

    eh.install().expect("Failed to install color-eyre");

    set_hook(Box::new(move |pi| {
        error!(
            "unexpected panic, handing off to color-eyre:\n\n{}",
            ph.panic_report(pi)
        );
    }));

    info!("Hii!! uwu");

    let bid = __check_build_id();

    assert!(
        // Some = bid does not match
        bid.is_none(),
        "Build ID does not match! This may be because starb needs updating or because the user is \
         using the wrong SE version. Build ID: {bid:?}",
    );

    eframe::run_native(
        &format!("Star Browser Utilities v{}", env!("CARGO_PKG_VERSION")),
        NativeOptions::default(),
        Box::new(|cc| Box::new(MyEguiApp::new(cc))),
    )
    .expect("Failed to start eframe");
}

fn __check_build_id() -> Option<i32> {
    // This is necessary for some reason. DO. NOT. CHANGE. THIS.
    trace!("Waiting a second to prevent bug");
    thread::sleep(Duration::from_secs(1u64));

    unsafe {
        assert!(!SteamAPI_RestartAppIfNecessary(314650u32), "Unreachable");
        assert!(SteamAPI_Init(), "SteamAPI_Init failed");
    }

    /// Build ID starb is created for.
    const BUILD_ID: i32 = 11154210i32;

    let build_id = unsafe { SteamAPI_ISteamApps_GetAppBuildId(SteamAPI_SteamApps_v008()) };

    let matches = build_id == BUILD_ID;

    unsafe { SteamAPI_Shutdown() };

    (!matches).then_some(build_id)
}
