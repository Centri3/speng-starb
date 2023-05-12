use crate::app::StarApp;
use eframe::CreationContext;
use eframe::Storage;
use egui::Context;
use egui::Frame;
use eyre::Result;

pub trait Plugin {
    /// Load the plugin.
    fn load(&self, cc: &CreationContext<'_>) -> Result<()>;

    /// Same as `update`, but called when the app adds plugins to the GUI. Use
    /// this to add arguments and stuff.
    fn add_update(&self, _app: &mut StarApp, _ctx: &Context, _frame: &mut Frame) {}

    /// Called when [`StarApp`]'s `update` method is called.
    fn update(&self, _app: &mut StarApp, _ctx: &Context, _frame: &mut Frame) {}

    /// Called when [`StarApp`]'s `save` method is called.
    fn save(&self, _app: &mut StarApp, _storage: &mut dyn Storage) {}

    /// Called when [`StarApp`]'s `on_close_event` method is called.
    fn on_close_event(&self, _app: &mut StarApp) {}

    /// Called when [`StarApp`]'s `on_exit` method is called.
    fn on_exit(&self, _app: &mut StarApp) {}
}
