use eframe::{App, CreationContext, Frame};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
#[serde(default)]
pub struct StarbApp {}

impl Default for StarbApp {
    fn default() -> Self {
        Self {}
    }
}

impl StarbApp {
    pub fn new(cc: &CreationContext) -> Self {
        if let Some(storage) = cc.storage {
            return eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
        }

        Self::default()
    }
}

impl App for StarbApp {
    fn update(&mut self, ctx: &egui::Context, frame: &mut Frame) {}
}
