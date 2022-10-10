use crate::handler::{self, CompactPatch, EsiFilter, Handler, NoMaxSearchRadius, Reason};
use eframe::{glow, App, Frame};
use egui::{Align, CentralPanel, Direction, Layout, Slider, TopBottomPanel, Window};

pub struct StarApp {
    pub handler: Handler,
    pub no_max_search_radius: (Option<NoMaxSearchRadius>, bool),
    pub no_max_search_radius_settings: (bool, bool, f32),
    pub no_search_locking: (Option<CompactPatch>, bool),
    pub accurate_temp_filter: (Option<CompactPatch>, bool),
    pub esi_filter: (Option<EsiFilter>, bool),
    pub esi_filter_settings: (f32, f32),
    pub chthonia_filter: (Option<CompactPatch>, bool),
    pub allowed_to_close: bool,
    pub show_confirmation_dialog: bool,
}

impl StarApp {
    pub fn new() -> Self {
        let no_search_locking_data = handler::NO_SEARCH_LOCKING_DATA.to_vec();
        let no_search_locking_data = vec![
            (
                no_search_locking_data[0usize].0,
                no_search_locking_data[0usize].1.to_vec(),
                no_search_locking_data[0usize].2.to_vec(),
            ),
            (
                no_search_locking_data[1usize].0,
                no_search_locking_data[1usize].1.to_vec(),
                no_search_locking_data[1usize].2.to_vec(),
            ),
            (
                no_search_locking_data[2usize].0,
                no_search_locking_data[2usize].1.to_vec(),
                no_search_locking_data[2usize].2.to_vec(),
            ),
        ];

        let accurate_temp_filter_data = vec![(
            handler::ACCURATE_TEMP_FILTER_DATA.0,
            handler::ACCURATE_TEMP_FILTER_DATA.1.to_vec(),
            handler::ACCURATE_TEMP_FILTER_DATA.2.to_vec(),
        )];

        let chthonia_filter = vec![(
            handler::CHTHONIA_FILTER_DATA.0,
            handler::CHTHONIA_FILTER_DATA.1.to_vec(),
            handler::CHTHONIA_FILTER_DATA.2.to_vec(),
        )];

        let handler = Handler::new();

        match handler.reason.is_none() {
            true => StarApp {
                handler,
                no_max_search_radius: (Some(NoMaxSearchRadius {}), false),
                no_max_search_radius_settings: (false, false, 0.0f32),
                no_search_locking: (Some(CompactPatch::new(no_search_locking_data)), false),
                accurate_temp_filter: (Some(CompactPatch::new(accurate_temp_filter_data)), false),
                esi_filter: (Some(EsiFilter::new(&handler.clone())), false),
                esi_filter_settings: (0.990f32, 1.0f32),
                chthonia_filter: (Some(CompactPatch::new(chthonia_filter)), false),
                allowed_to_close: false,
                show_confirmation_dialog: false,
            },
            false => StarApp {
                handler,
                no_max_search_radius: (None, false),
                no_max_search_radius_settings: (false, false, 0.0f32),
                no_search_locking: (None, false),
                accurate_temp_filter: (None, false),
                esi_filter: (None, false),
                esi_filter_settings: (0.990f32, 0.10f32),
                chthonia_filter: (None, false),
                allowed_to_close: false,
                show_confirmation_dialog: false,
            },
        }
    }
}

impl App for StarApp {
    fn update(&mut self, ctx: &egui::Context, frame: &mut Frame) {
        ctx.set_pixels_per_point(1.5f32);

        TopBottomPanel::top("title_panel").show(ctx, |ui| {
            ui.with_layout(Layout::top_down(Align::Center), |ui| {
                egui::global_dark_light_mode_switch(ui);

                ui.label("Star Browser Utilities");
            });
        });

        CentralPanel::default().show(ctx, |ui| {
            ui.with_layout(Layout::bottom_up(Align::Center), |ui| {
                ui.label(format!(
                    "Handler\nreason: {:?}\nhandle: {:?}\npid: {}\nbase_address: {:x}",
                    self.handler.reason,
                    self.handler.handle,
                    self.handler.pid,
                    self.handler.base_address,
                ));
            });
        });

        if self.handler.reason.is_none() && self.handler.still_open() {
            TopBottomPanel::top("patches_panel").show(ctx, |ui| {
                ui.with_layout(Layout::top_down(Align::LEFT), |ui| {
                    ui.checkbox(&mut self.no_max_search_radius.1, "No Max Search Radius")
                        .on_hover_text(
                            "Remove the 100pc (326.16ly) search radius limit, or set your \
                             own!\n\nWARNING: Setting this too high while searching for \
                             rarer\nstars (Neutron stars, Black holes, etc) will lag the game, \
                             and\npossibly crash it.",
                        );

                    match self.no_max_search_radius.1 {
                        true => self
                            .no_max_search_radius
                            .0
                            .as_ref()
                            .unwrap()
                            .enable(self.no_max_search_radius_settings, &self.handler),
                        false => self
                            .no_max_search_radius
                            .0
                            .as_ref()
                            .unwrap()
                            .disable(self.no_max_search_radius_settings, &self.handler),
                    }

                    if self.no_max_search_radius.1 {
                        ui.checkbox(&mut self.no_max_search_radius_settings.0, "Custom Max")
                            .on_hover_text(
                                "Set your own custom max search radius, up to 100,000ly",
                            );

                        if self.no_max_search_radius_settings.0 {
                            ui.checkbox(&mut self.no_max_search_radius_settings.1, "Use Parsecs")
                                .on_hover_text(
                                    "Use parsecs instead of light years, divides the max search \
                                     radius by ~3.26156",
                                );

                            ui.add(
                                Slider::new(
                                    &mut self.no_max_search_radius_settings.2,
                                    0.0f32..=match self.no_max_search_radius_settings.1 {
                                        true => 30660.17f32,
                                        false => 100000.0f32,
                                    },
                                )
                                .suffix(match self.no_max_search_radius_settings.1 {
                                    true => "pc",
                                    false => "ly",
                                })
                                .text("Max Search Radius")
                                .logarithmic(true),
                            );
                        }
                    }

                    ui.separator();

                    if ui
                        .checkbox(&mut self.no_search_locking.1, "No Search Locking")
                        .on_hover_text(
                            "Tries to fix SE's search button locking occasionally on newer \
                             versions.\n\nNOTE: This doesn't entirely patch it, but uses a much \
                             better method which allows\nyou to press stop and clear to fix it, \
                             rather than needing to input StarBrowserReset\nin the console.",
                        )
                        .clicked()
                    {
                        match self.no_search_locking.1 {
                            true => self
                                .no_search_locking
                                .0
                                .as_ref()
                                .unwrap()
                                .enable(&self.handler),
                            false => self
                                .no_search_locking
                                .0
                                .as_ref()
                                .unwrap()
                                .disable(&self.handler),
                        }
                    }

                    ui.separator();

                    if ui
                        .checkbox(
                            &mut self.accurate_temp_filter.1,
                            "Accurate Temperature Filter",
                        )
                        .on_hover_text(
                            "The Star browser currently uses Current Temperature at\ntime January \
                             1st 2000, 12:00:00, this forces it to use Average Temperature \
                             instead.\n\nNOTE: This can be even less accurate at times than \
                             current temperature, but is\nusually much closer to what your \
                             filters are.",
                        )
                        .clicked()
                    {
                        match self.accurate_temp_filter.1 {
                            true => self
                                .accurate_temp_filter
                                .0
                                .as_ref()
                                .unwrap()
                                .enable(&self.handler),
                            false => self
                                .accurate_temp_filter
                                .0
                                .as_ref()
                                .unwrap()
                                .disable(&self.handler),
                        }
                    }

                    ui.separator();

                    ui.checkbox(&mut self.esi_filter.1, "ESI Filter")
                        .on_hover_text("Adds the long requested ESI Filter. That's it.");

                    if self.esi_filter.1 {
                        ui.add(
                            Slider::new(&mut self.esi_filter_settings.0, 0.0f32..=1.0f32)
                                .text("Minimum ESI")
                                .min_decimals(3usize),
                        );

                        ui.add(
                            Slider::new(&mut self.esi_filter_settings.1, 0.0f32..=1.0f32)
                                .text("Maximum ESI")
                                .min_decimals(3usize),
                        );

                        // This is disgusting, and buggy
                        if self.esi_filter_settings.1 < self.esi_filter_settings.0 {
                            self.esi_filter_settings.0 = self.esi_filter_settings.1;
                        }
                    }

                    // There's definitely a better way to do this, but the impact is negligible
                    match self.esi_filter.1 {
                        true => self
                            .esi_filter
                            .0
                            .as_ref()
                            .unwrap()
                            .enable(self.esi_filter_settings, &self.handler),
                        false => self.esi_filter.0.as_ref().unwrap().disable(&self.handler),
                    }

                    ui.separator();

                    if ui
                        .checkbox(&mut self.chthonia_filter.1, "Chthonia Filter")
                        .on_hover_text(
                            "Adds chthonia as a bulk-class filter. The chthonia bulk-class was \
                             meant to be\nremoved long ago, but due to some bug,any gas giant \
                             with >25% helium in\nits composition is a chthonia. This lets you \
                             search for them again, which you haven't\nbeen able to since \
                             0.990.35 (in vanilla, anyway)",
                        )
                        .clicked()
                    {
                        match self.chthonia_filter.1 {
                            true => self
                                .chthonia_filter
                                .0
                                .as_ref()
                                .unwrap()
                                .enable(&self.handler),
                            false => self
                                .chthonia_filter
                                .0
                                .as_ref()
                                .unwrap()
                                .disable(&self.handler),
                        }
                    }
                });
            });
        } else {
            match self.handler.reason.as_ref() {
                Some(Reason::NotFound) => Window::new("Failed to find SE!")
                    .collapsible(false)
                    .resizable(false)
                    .show(ctx, |ui| {
                        ui.label(format!(
                            "Failed to find SE in the list of open processes! Please restart this \
                             program once you open it.",
                        ));
                    }),
                Some(Reason::FailedToOpen) => Window::new("Failed to open a handle to SE!")
                    .collapsible(false)
                    .resizable(false)
                    .show(ctx, |ui| {
                        ui.label(format!(
                            "Failed to open a handle to SE! Why did this happen? I have no idea! \
                             I'm just handling it incase it happens! Please report if you see \
                             this.",
                        ));
                    }),
                Some(Reason::WrongVersion) => Window::new("Wrong Version!")
                    .collapsible(false)
                    .resizable(false)
                    .show(ctx, |ui| {
                        ui.label(format!(
                            "You're not on the GR public beta! Please switch your branch to \
                             CurrentBeta.",
                        ));
                    }),
                Some(Reason::TooManyInstances) => Window::new("Too Many Instances!")
                    .collapsible(false)
                    .resizable(false)
                    .show(ctx, |ui| {
                        ui.label(format!(
                            "You already have another instance of this program open! Please close \
                             the other instance.",
                        ));
                    }),
                None => Window::new("SE Closed!")
                    .collapsible(false)
                    .resizable(false)
                    .show(ctx, |ui| {
                        ui.label(format!(
                            "You closed SE! Please restart this program once you reopen it.",
                        ));
                    }),
            };
        }

        // <https://github.com/emilk/egui/blob/master/examples/confirm_exit/src/main.rs> with minor edits
        if self.show_confirmation_dialog {
            egui::Window::new("Please confirm exit")
                .open(&mut self.show_confirmation_dialog)
                .collapsible(false)
                .resizable(false)
                .show(ctx, |ui| {
                    ui.with_layout(Layout::top_down(Align::Center), |ui| {
                        ui.label("Exiting will disable all patches. Are you sure?")
                    });

                    ui.horizontal(|ui| {
                        ui.with_layout(
                            Layout::centered_and_justified(Direction::LeftToRight),
                            |ui| {
                                if ui.button("Exit").clicked() {
                                    self.allowed_to_close = true;
                                    frame.close();
                                }
                            },
                        );
                    });
                });
        }
    }

    fn on_close_event(&mut self) -> bool {
        self.show_confirmation_dialog = true;
        self.allowed_to_close
    }

    fn on_exit(&mut self, _: Option<&glow::Context>) {
        // Cleaning up once we're done
        self.no_max_search_radius
            .0
            .as_ref()
            .unwrap()
            .disable(self.no_max_search_radius_settings, &self.handler);
        self.no_search_locking
            .0
            .as_ref()
            .unwrap()
            .disable(&self.handler);
        self.accurate_temp_filter
            .0
            .as_ref()
            .unwrap()
            .disable(&self.handler);
        self.esi_filter.0.as_ref().unwrap().disable(&self.handler);
        self.chthonia_filter
            .0
            .as_ref()
            .unwrap()
            .disable(&self.handler);

        // Close handles. No clue what happens if I don't do this but no reason why I shouldn't (even if it crashes, it's already closing!)
        self.esi_filter.0.as_ref().unwrap().close(&self.handler);
        self.handler.close();
    }
}
