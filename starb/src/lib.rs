pub mod app;
pub mod plugin;
pub mod plugins;

use app::StarApp;
use color_eyre::config::HookBuilder;
use eframe::NativeOptions;
use std::env::current_exe;
use std::fs::File;
use std::panic::set_hook;
use std::ptr::addr_of_mut;
use std::sync::Arc;
use std::thread;
use std::time::Duration;
use steamworks_sys::SteamAPI_ISteamApps_GetAppBuildId;
use steamworks_sys::SteamAPI_Init;
use steamworks_sys::SteamAPI_RestartAppIfNecessary;
use steamworks_sys::SteamAPI_Shutdown;
use steamworks_sys::SteamAPI_SteamApps_v008;
use tracing::debug;
use tracing::error;
use tracing::info;
use tracing::trace;
use tracing::Level;
use tracing_subscriber::fmt::format::FmtSpan;
use windows_sys::Win32::Foundation::HMODULE;
use windows_sys::Win32::Foundation::LPARAM;
use windows_sys::Win32::System::SystemServices::DLL_PROCESS_ATTACH;
use windows_sys::Win32::System::SystemServices::UNICODE_STRING_MAX_CHARS;
use windows_sys::Win32::System::Threading::GetCurrentProcessId;
use windows_sys::Win32::UI::WindowsAndMessaging::EnumWindows;
use windows_sys::Win32::UI::WindowsAndMessaging::GetClassNameA;
use windows_sys::Win32::UI::WindowsAndMessaging::GetClassNameW;
use windows_sys::Win32::UI::WindowsAndMessaging::GetWindowLongA;
use windows_sys::Win32::UI::WindowsAndMessaging::GetWindowThreadProcessId;
use windows_sys::Win32::UI::WindowsAndMessaging::IsWindowVisible;
use windows_sys::Win32::UI::WindowsAndMessaging::GWLP_ID;

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
        .with_max_level(Level::DEBUG)
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

    // This is necessary for some reason. DO. NOT. CHANGE. THIS. Basically, Steam
    // API is FUCKED for SE. So, we wait until it stops using it to use it
    // ourselves.
    loop {
        let mut found_se = false;

        unsafe {
            assert_ne!(
                EnumWindows(
                    Some(__check_if_window_is_opened),
                    addr_of_mut!(found_se) as isize,
                ),
                0i32,
                "EnumWindows failed"
            );
        };

        if found_se {
            trace!("Found SE window; We can begin!");
            break;
        }

        trace!("Did not find SE window. Retrying in 100ms...");

        // Don't use all of the CPU
        thread::sleep(Duration::from_millis(100u64));
    }

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
        Box::new(|cc| Box::new(StarApp::new(cc))),
    )
    .expect("Failed to start eframe");
}

unsafe extern "system" fn __check_if_window_is_opened(hwnd: isize, found: LPARAM) -> i32 {
    let mut pid = 0u32;
    unsafe { GetWindowThreadProcessId(hwnd, &mut pid) };

    if pid == unsafe { GetCurrentProcessId() } {
        let mut name = [0u16; UNICODE_STRING_MAX_CHARS as usize];
        let name_len = unsafe { GetClassNameW(hwnd, name.as_mut_ptr(), name.len() as i32) };
        let name = String::from_utf16_lossy(&name[..name_len as usize]);

        // Ignore splash screen. IsWindowVisible is necessary due to... Invisible
        // windows??? IDK
        if name != "SE Splash" && unsafe { IsWindowVisible(hwnd) } == 1i32 {
            // SAFETY: We must uphold that this is the only reference to found. That's easy!
            unsafe { (found as *mut bool).write(true) };
        }
    }

    i32::from(true)
}

fn __check_build_id() -> Option<i32> {
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
