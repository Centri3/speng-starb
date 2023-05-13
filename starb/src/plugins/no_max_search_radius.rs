use crate::app::StarApp;
use crate::plugin::Plugin;
use crate::plugin::PluginPass;
use crate::utils::base;
use crate::utils::write;
use eframe::CreationContext;
use eframe::Frame;
use eframe::Storage;
use egui::Context;
use egui::Ui;
use eyre::Result;
use serde::Deserialize;
use serde::Serialize;
use tracing::instrument;

const PLUGIN_KEY: &str = "no_max_search_radius";

#[derive(Deserialize, Serialize)]
pub struct NoMaxSearchRadius(bool);

#[allow(clippy::derivable_impls)]
impl Default for NoMaxSearchRadius {
    fn default() -> Self {
        Self(false)
    }
}

impl Plugin for NoMaxSearchRadius {
    #[instrument(skip(cc))]
    fn load(cc: &CreationContext<'_>) -> Result<Self>
    where
        Self: Sized,
    {
        let no_max_search_radius =
            eframe::get_value::<Self>(cc.storage.expect("Probably unreachable?"), PLUGIN_KEY)
                .unwrap_or_default();

        // TODO: Don't do this here. Quick hotfix
        unsafe {
            if no_max_search_radius.0 {
                write::<u8>(base().byte_offset(0x3EFBA0isize).cast(), 0xEBu8)
            }
            else {
                write::<u8>(base().byte_offset(0x3EFBA0isize).cast(), 0x76u8)
            }
        }
        .expect("Failed to update ):");

        Ok(no_max_search_radius)
    }

    fn pass(&self) -> PluginPass {
        PluginPass::Late
    }

    fn name(&self) -> String {
        "No Max Search Radius".to_owned()
    }

    fn priority(&self) -> Option<usize> {
        Some(1usize)
    }

    #[instrument(skip(self, _app, _ctx, _frame, ui))]
    fn add_plugin(&mut self, _app: &mut StarApp, _ctx: &Context, _frame: &mut Frame, ui: &mut Ui) {
        if ui
            .checkbox(&mut self.0, "Enabled")
            .on_hover_text("Uncap the Star browser's search radius")
            .clicked()
        {
            unsafe {
                if self.0 {
                    write::<u8>(base().byte_offset(0x3EFBA0isize).cast(), 0xEBu8)
                }
                else {
                    write::<u8>(base().byte_offset(0x3EFBA0isize).cast(), 0x76u8)
                }
            }
            .expect("Failed to update ):");
        }
    }

    fn add_context(&mut self, _app: &mut StarApp, _ctx: &Context, _frame: &mut Frame, ui: &mut Ui) {
        ui.label(format!(
            "Max search radius is currently {}",
            if self.0 { "100.0f64" } else { "UNLIMITED" }
        ));
    }

    fn save(&mut self, _app: &mut StarApp, storage: &mut dyn Storage) {
        eframe::set_value(storage, PLUGIN_KEY, &self);
    }
}
