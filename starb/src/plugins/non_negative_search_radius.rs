//! TODO

use crate::app::StarApp;
use crate::plugin::Plugin;
use crate::plugin::PluginPass;
use crate::utils::base;
use eframe::CreationContext;
use eframe::Frame;
use eframe::Storage;
use egui::Context;
use egui::Ui;
use eyre::Result;
use serde::Deserialize;
use serde::Serialize;
use std::mem::forget;
use std::slice;
use tracing::instrument;

const PLUGIN_KEY: &str = "non_negative_search_radius";

/// .0 = enabled, .1 = behavior
#[derive(Deserialize, Serialize)]
pub struct NonNegativeSearchRadius(bool, bool);

impl Default for NonNegativeSearchRadius {
    fn default() -> Self {
        Self(false, true)
    }
}

impl Plugin for NonNegativeSearchRadius {
    #[instrument(skip(cc))]
    fn load(cc: &CreationContext<'_>) -> Result<Self>
    where
        Self: Sized,
    {
        let non_negative_search_radius =
            eframe::get_value(cc.storage.expect("Probably unreachable?"), PLUGIN_KEY)
                .unwrap_or_default();

        Ok(non_negative_search_radius)
    }

    fn pass(&self) -> PluginPass {
        PluginPass::Late
    }

    fn name(&self) -> String {
        "Non Negative Search Radius".to_owned()
    }

    fn priority(&self) -> Option<usize> {
        Some(3usize)
    }

    #[instrument(skip(self, _app, ctx, _frame, ui))]
    fn add_plugin(&mut self, _app: &mut StarApp, ctx: &Context, _frame: &mut Frame, ui: &mut Ui) {
        ui.checkbox(&mut self.0, "Enabled")
            .on_hover_text("Disallow negative search radii\n\nThis is a very minor bugfix.");

        if self.0 {
            ui.checkbox(&mut self.1, "Use absolute value")
                .on_hover_text(
                    "Use the absolute value of the search radius instead of setting it to 0.",
                );

            let search_radius = unsafe { base().byte_offset(0x10457B0isize).cast::<f64>() };
            let old = unsafe { search_radius.read() };

            if old.is_sign_negative() {
                let new = if self.1 { old.abs() } else { 0.0f64 };

                unsafe { search_radius.write(new) };

                let str_ptr =
                    unsafe { base().byte_offset(0x1046F08isize).cast::<*mut u16>().read() };
                let str_len_ptr = unsafe { base().byte_offset(0x1046F18isize).cast::<usize>() };
                let str_len = unsafe { str_len_ptr.read() };
                let str = String::from_utf16(unsafe { slice::from_raw_parts(str_ptr, str_len) })
                    .expect("String was not valid utf-16. Shame on you, vladimir.");

                let (new_str, new_str_len, _) = new
                    .to_string()
                    .encode_utf16()
                    .collect::<Vec<u16>>()
                    .into_raw_parts();

                // TODO: We could probably do this better.
                #[allow(clippy::forget_copy)]
                forget(new_str);

                unsafe { (str_ptr as usize as *mut usize).write(new_str as usize) }
                unsafe { str_len_ptr.write(new_str_len) };
            }

            // TODO: We could probably do this better.
            ctx.request_repaint();
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
