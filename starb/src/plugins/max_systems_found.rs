use crate::plugin::Plugin;
use crate::plugin::PluginPass;
use crate::utils::base;
use crate::utils::read;
use crate::utils::sys_folder;
use crate::utils::write;
use eframe::CreationContext;
use eframe::Frame;
use egui::Context;
use egui::Ui;
use eyre::Result;
use std::fs;
use tracing::info;
use tracing::instrument;
use tracing::warn;

#[derive(Clone)]
pub struct MaxSystemsFound(u32);

impl Plugin for MaxSystemsFound {
    #[instrument]
    fn load(_: &CreationContext<'_>) -> Result<Self>
    where
        Self: Sized,
    {
        let max_systems_found = fs::read_to_string(sys_folder()?.join("STARB_MAXSYSTEMSFOUND"))?
            .trim()
            .parse::<u32>()?;

        let fir = (base() + 0x3F1531) as *mut u32;
        let sec = (base() + 0x3F1549) as *mut u32;

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
            write(fir, max_systems_found)?;
            write(sec, max_systems_found)?;
        }

        info!("Changed max systems found to {max_systems_found}.");

        Ok(Self(max_systems_found))
    }

    fn pass(&self) -> PluginPass {
        PluginPass::Early
    }

    fn name(&self) -> String {
        "Custom max systems found".to_owned()
    }

    fn priority(&self) -> Option<usize> {
        Some(1usize)
    }

    fn add_context(&self, ctx: &Context, frame: &mut Frame, ui: &mut Ui) {
        ui.label(format!("Max systems found is currently {}", self.0));
    }
}
