use crate::app::StarApp;
use crate::plugin::Plugin;
use crate::plugin::PluginPass;
use crate::utils::base;
use crate::utils::read;
use crate::utils::write;
use eframe::CreationContext;
use eframe::Frame;
use eframe::Storage;
use egui::Context;
use egui::Slider;
use egui::Ui;
use eyre::Result;
use serde::Deserialize;
use serde::Serialize;
use tracing::info;
use tracing::instrument;
use tracing::warn;

const PLUGIN_KEY: &str = "no_max_systems_found";

/// .0 = requested, .1 = current
#[derive(Clone, Deserialize, Serialize)]
pub struct NoMaxSystemsFound(u32, u32);

impl Default for NoMaxSystemsFound {
    fn default() -> Self {
        Self(10000u32, 10000u32)
    }
}

impl Plugin for NoMaxSystemsFound {
    #[instrument(skip(cc))]
    fn load(cc: &CreationContext<'_>) -> Result<Self>
    where
        Self: Sized,
    {
        let no_max_systems_found =
            eframe::get_value::<Self>(cc.storage.expect("Probably unreachable?"), PLUGIN_KEY)
                .unwrap_or_default();

        if no_max_systems_found.0 > 1000000u32 {
            warn!("NO MAX SYSTEMS FOUND IS ABOVE 1000000. UH OH!");
        }

        let fir = unsafe { base().byte_offset(0x3F1531isize).cast::<u32>() };
        let sec = unsafe { base().byte_offset(0x3F1549isize).cast::<u32>() };

        unsafe {
            assert_eq!(
                fir.read(),
                sec.read(),
                "These are not equal! THIS IS IMPOSSIBLE. WRONG SE VERSION!"
            );
        }

        // SAFETY: THIS IS UNSAFE.
        if unsafe { read(fir)? != 10000u32 } || unsafe { read(sec)? != 10000u32 } {
            warn!("Either of fir or sec are not 10000! This exe is likely modifed, but that's ok.");
        }

        // SAFETY: The above check should be enough, UNLESS both HAPPEN to be the same
        // SOMEHOW. I cannot stress enough how rare this would be (unless they're both
        // 0xCC...?).
        unsafe {
            write(fir, no_max_systems_found.0)?;
            write(sec, no_max_systems_found.0)?;
        }

        info!("Changed max systems found to {}.", no_max_systems_found.0);

        Ok(no_max_systems_found)
    }

    fn pass(&self) -> PluginPass {
        PluginPass::Early
    }

    fn name(&self) -> String {
        "No Max Systems Found".to_owned()
    }

    fn priority(&self) -> Option<usize> {
        Some(0usize)
    }

    fn add_plugin(&mut self, app: &mut StarApp, _ctx: &Context, _frame: &mut Frame, ui: &mut Ui) {
        if ui
            .add(Slider::new(&mut self.0, 0u32..=1000000u32).logarithmic(true))
            .on_hover_text(
                "What to set the max number of systems the Star browser will search.\n\nDefault: \
                 10000",
            )
            .changed()
        {
            app.requires_restart(
                &self.name(),
                &"Max systems found cannot be modified after SE has already started.",
            );
        }

        if self.0 == 1000000u32 {
            ui.label("Values above 1000000 are very unstable. They cannot to be set.");
        }
        else if self.0 > 100000u32 {
            ui.label("Values above 100000 are both unnecessary and difficult to run.");
        }
    }

    fn add_context(&mut self, _app: &mut StarApp, _ctx: &Context, _frame: &mut Frame, ui: &mut Ui) {
        ui.label(format!("Requested max systems found is {}", self.0));
        ui.label(format!("Max systems found is currently {}", self.1));
    }

    fn save(&mut self, _app: &mut StarApp, storage: &mut dyn Storage) {
        eframe::set_value(storage, PLUGIN_KEY, &self);
    }
}
