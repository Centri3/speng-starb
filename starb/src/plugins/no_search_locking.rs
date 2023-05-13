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

const PLUGIN_KEY: &str = "no_search_locking";

#[derive(Deserialize, Serialize)]
pub struct NoSearchLocking(bool);

#[allow(clippy::derivable_impls)]
impl Default for NoSearchLocking {
    fn default() -> Self {
        Self(false)
    }
}

impl Plugin for NoSearchLocking {
    #[instrument(skip(cc))]
    fn load(cc: &CreationContext<'_>) -> Result<Self>
    where
        Self: Sized,
    {
        let no_search_locking =
            eframe::get_value::<Self>(cc.storage.expect("Probably unreachable?"), PLUGIN_KEY)
                .unwrap_or_default();

        // TODO: Don't do this here. Quick hotfix
        unsafe {
            if no_search_locking.0 {
                // Prevent flashing on GUI
                write::<[u8; 3usize]>(base().byte_offset(0x3EC50Aisize).cast(), [
                    0x8Bu8, 0xC2u8, 0x90u8,
                ])?;
                // nop 6, the actual fix
                write::<[u8; 6usize]>(base().byte_offset(0x3EFB2Cisize).cast(), [
                    0x66u8, 0x0Fu8, 0x1Fu8, 0x44u8, 0x00u8, 0x00u8,
                ])?;
                write::<[u8; 6usize]>(base().byte_offset(0x3EFD4Eisize).cast(), [
                    0x66u8, 0x0Fu8, 0x1Fu8, 0x44u8, 0x00u8, 0x00u8,
                ])?;
            }
            else {
                write::<[u8; 3usize]>(base().byte_offset(0x3EC50Aisize).cast(), [
                    0x0Fu8, 0x44u8, 0xC2u8,
                ])?;
                write::<[u8; 6usize]>(base().byte_offset(0x3EFB2Cisize).cast(), [
                    0x0Fu8, 0x85u8, 0x82u8, 0x00u8, 0x00u8, 0x00u8,
                ])?;
                write::<[u8; 6usize]>(base().byte_offset(0x3EFD4Eisize).cast(), [
                    0x0Fu8, 0x85u8, 0xBAu8, 0x01u8, 0x00u8, 0x00u8,
                ])?;
            }
        }

        Ok(no_search_locking)
    }

    fn pass(&self) -> PluginPass {
        PluginPass::Late
    }

    fn name(&self) -> String {
        "No Search Locking".to_owned()
    }

    fn priority(&self) -> Option<usize> {
        Some(2usize)
    }

    #[instrument(skip(self, _app, _ctx, _frame, ui))]
    fn add_plugin(&mut self, _app: &mut StarApp, _ctx: &Context, _frame: &mut Frame, ui: &mut Ui) {
        if ui
            .checkbox(&mut self.0, "Enabled")
            .on_hover_text(
                "Prevent the Star browser from occasionally locking after a search \
                 concludes.\n\nThis is a MAJOR bugfix; It's highly recommended to keep this on.",
            )
            .clicked()
        {
            // Weird try block
            || -> Result<()> {
                unsafe {
                    if self.0 {
                        // Prevent flashing on GUI
                        write::<[u8; 3usize]>(base().byte_offset(0x3EC50Aisize).cast(), [
                            0x8Bu8, 0xC2u8, 0x90u8,
                        ])?;
                        // nop 6, the actual fix
                        write::<[u8; 6usize]>(base().byte_offset(0x3EFB2Cisize).cast(), [
                            0x66u8, 0x0Fu8, 0x1Fu8, 0x44u8, 0x00u8, 0x00u8,
                        ])?;
                        write::<[u8; 6usize]>(base().byte_offset(0x3EFD4Eisize).cast(), [
                            0x66u8, 0x0Fu8, 0x1Fu8, 0x44u8, 0x00u8, 0x00u8,
                        ])?;
                    }
                    else {
                        write::<[u8; 3usize]>(base().byte_offset(0x3EC50Aisize).cast(), [
                            0x0Fu8, 0x44u8, 0xC2u8,
                        ])?;
                        write::<[u8; 6usize]>(base().byte_offset(0x3EFB2Cisize).cast(), [
                            0x0Fu8, 0x85u8, 0x82u8, 0x00u8, 0x00u8, 0x00u8,
                        ])?;
                        write::<[u8; 6usize]>(base().byte_offset(0x3EFD4Eisize).cast(), [
                            0x0Fu8, 0x85u8, 0xBAu8, 0x01u8, 0x00u8, 0x00u8,
                        ])?;
                    }

                    Ok(())
                }
            }()
            .expect("Failed to update ):");
        }
    }

    fn save(&mut self, _app: &mut StarApp, storage: &mut dyn Storage) {
        eframe::set_value(storage, PLUGIN_KEY, &self);
    }
}
