use crate::plugin::Plugin;
use crate::plugins::max_systems_found::MaxSystemsFound;
use eframe::App;
use eframe::CreationContext;
use eframe::Frame;
use eframe::APP_KEY;
use egui::CentralPanel;
use egui::Context;
use egui::TopBottomPanel;
use serde::Deserialize;
use serde::Serialize;
use std::ptr::addr_of_mut;
use std::thread;
use std::time::Duration;
use steamworks_sys::SteamAPI_ISteamApps_GetAppBuildId;
use steamworks_sys::SteamAPI_Init;
use steamworks_sys::SteamAPI_RestartAppIfNecessary;
use steamworks_sys::SteamAPI_Shutdown;
use steamworks_sys::SteamAPI_SteamApps_v008;
use tracing::info;
use tracing::trace;
use windows_sys::Win32::Foundation::LPARAM;
use windows_sys::Win32::System::SystemServices::UNICODE_STRING_MAX_CHARS;
use windows_sys::Win32::System::Threading::GetCurrentProcessId;
use windows_sys::Win32::UI::WindowsAndMessaging::EnumWindows;
use windows_sys::Win32::UI::WindowsAndMessaging::GetClassNameW;
use windows_sys::Win32::UI::WindowsAndMessaging::GetWindowThreadProcessId;
use windows_sys::Win32::UI::WindowsAndMessaging::IsWindowVisible;

type Plugins = Vec<Box<dyn Plugin>>;

#[derive(Default)]
pub enum Tab {
    #[default]
    StarbPlugins,
    Filters,
    Context,
    CustomPlugins,
}

#[derive(Deserialize, Serialize)]
pub struct StarApp {
    #[serde(skip)]
    tab: Tab,
    #[serde(skip)]
    tab_internal: (bool, bool, bool, bool),
    #[serde(skip)]
    plugins: Plugins,
}

macro __plugins {
    ($($cc:expr)?) => { Vec::<Box<dyn Plugin>>::new() },
    ($cc:expr, $($plugin:ident),* $(,)?) => {{
        let mut plugins = Vec::<Box<dyn Plugin>>::new();

        $(
            plugins.push(Box::new(MaxSystemsFound::load($cc).unwrap_or_else(|e| {
                panic!("Failed to load `{}`: {e}", stringify!($plugin))
            })));
        )*

        plugins
    }}
}

impl StarApp {
    #[must_use]
    #[allow(clippy::vec_init_then_push)]
    pub fn new(cc: &CreationContext<'_>) -> Self {
        // TODO: Find and add custom early/late plugins.

        let mut early_plugins = __plugins! {
            cc,
            MaxSystemsFound,
        };

        info!("Early plugins:");

        __print_plugins(&early_plugins);

        info!("Waiting for SE's main window to open...");

        // This is necessary for some reason. DO. NOT. CHANGE. THIS. Basically, Steam
        // API is FUCKED for SE. So, we wait until it stops using it to use it
        // ourselves.
        loop {
            let mut found_se = false;
            let mut times = 0i32;

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

            times += 1i32;
            trace!(times, "Did not find SE window. Retrying in 100ms...");

            // Don't use all of the CPU
            thread::sleep(Duration::from_millis(100u64));
        }

        let bid = __check_build_id();

        assert!(
            // Some = bid does not match
            bid.is_none(),
            "Build ID does not match! This may be because starb needs updating or because the \
             user is using the wrong SE version. Build ID: {bid:?}",
        );

        let mut late_plugins = __plugins! {};

        info!("Late plugins:");

        __print_plugins(&late_plugins);

        // TODO: if_chain?
        if let Some(storage) = cc.storage {
            if let Some(app) = eframe::get_value::<Self>(storage, APP_KEY) {
                return app.with_plugins(late_plugins);
            }
        }

        let mut plugins = vec![];
        plugins.append(&mut early_plugins);
        plugins.append(&mut late_plugins);

        Self {
            tab: Tab::StarbPlugins,
            // Spaghetti I think
            tab_internal: (true, false, false, false),
            plugins,
        }
    }

    #[must_use]
    pub fn with_plugins(self, plugins: Vec<Box<dyn Plugin>>) -> Self {
        Self { plugins, ..self }
    }
}

impl App for StarApp {
    fn update(&mut self, ctx: &Context, frame: &mut Frame) {
        ctx.set_pixels_per_point(1.5f32);

        TopBottomPanel::top("menu_bar").show(ctx, |ui| {
            ui.horizontal_centered(|ui| {
                // Oops! All spaghetti!
                //
                // TODO: Let's like, not? This is stupid
                if ui
                    .toggle_value(&mut self.tab_internal.0, "Plugins")
                    .clicked()
                {
                    // Spaghetti, but disallows disabling ALL tabs
                    if !self.tab_internal.0 {
                        self.tab_internal.0 = true;
                    }

                    self.tab = Tab::StarbPlugins;

                    self.tab_internal.1 = false;
                    self.tab_internal.2 = false;
                    self.tab_internal.3 = false;
                }
                else if ui
                    .toggle_value(&mut self.tab_internal.1, "Filters")
                    .clicked()
                {
                    // Spaghetti, but disallows disabling ALL tabs
                    if !self.tab_internal.1 {
                        self.tab_internal.1 = true;
                    }

                    self.tab = Tab::Filters;

                    self.tab_internal.0 = false;
                    self.tab_internal.2 = false;
                    self.tab_internal.3 = false;
                }
                else if ui
                    .toggle_value(&mut self.tab_internal.2, "Context")
                    .clicked()
                {
                    // Spaghetti, but disallows disabling ALL tabs
                    if !self.tab_internal.2 {
                        self.tab_internal.2 = true;
                    }

                    self.tab = Tab::Context;

                    self.tab_internal.0 = false;
                    self.tab_internal.1 = false;
                    self.tab_internal.3 = false;
                }
                else if ui
                    .toggle_value(&mut self.tab_internal.3, "Custom Plugins")
                    .clicked()
                {
                    // Spaghetti, but disallows disabling ALL tabs
                    if !self.tab_internal.3 {
                        self.tab_internal.3 = true;
                    }

                    self.tab = Tab::CustomPlugins;

                    self.tab_internal.0 = false;
                    self.tab_internal.1 = false;
                    self.tab_internal.2 = false;
                }
            })
        });

        // Context tab
        if matches!(self.tab, Tab::Context) {
            CentralPanel::default().show(ctx, |ui| {
                for plugin in &self.plugins {
                    plugin.add_context(ctx, frame, ui);
                }
            });
        }
        // Custom (user-made) plugins tab
        else if matches!(self.tab, Tab::CustomPlugins) {
            CentralPanel::default().show(ctx, |ui| {
                ui.vertical_centered_justified(|ui| {
                    ui.label("Coming soon...");
                })
            });
        }
    }

    // TODO: Disallow exiting. Ok not really, the user CAN edit, but warn them that
    // this may cause issues and tell them to use STARB_DONOTSTART instead.
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
        if name != "SE Splash"
            && name != "Winit Thread Event Target"
            && unsafe { IsWindowVisible(hwnd) } == 1i32
        {
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

fn __print_plugins(plugins: &Vec<Box<dyn Plugin>>) {
    for plugin in plugins {
        info!(name = plugin.name());
    }
}
