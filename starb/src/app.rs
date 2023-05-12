use crate::plugin::Plugin;
use eframe::App;
use eframe::CreationContext;
use eframe::Frame;
use eframe::APP_KEY;
use egui::CentralPanel;
use egui::Context;
use serde::Deserialize;
use serde::Serialize;

#[derive(Deserialize, Serialize)]
pub struct StarApp {
    #[serde(skip)]
    pub plugins: Option<Vec<Box<dyn Plugin>>>,
}

impl Default for StarApp {
    fn default() -> Self {
        Self { plugins: None }
    }
}

impl StarApp {
    #[must_use]
    pub fn new(cc: &CreationContext<'_>) -> Self {
        if let Some(storage) = cc.storage {
            return eframe::get_value(storage, APP_KEY).unwrap_or_default();
        }

        Self::default()
    }
}

impl App for StarApp {
    fn update(&mut self, ctx: &Context, frame: &mut Frame) {
        CentralPanel::default().show(ctx, |ui| {
            ui.heading("Hello World!");
        });
    }
}
