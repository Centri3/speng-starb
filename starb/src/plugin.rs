use crate::app::StarApp;
use eframe::CreationContext;
use eframe::Frame;
use eframe::Storage;
use egui::Context;
use eyre::Result;

#[derive(Debug)]
pub enum PluginPass {
    Early,
    Late,
}

pub trait Plugin {
    /// Load the plugin.
    fn load(cc: &CreationContext<'_>) -> Result<Self>
    where
        Self: Sized;

    /// When the plugin's loaded. `Early` is the first chance starb can get,
    /// `Late` is after SE's main window has opened.
    fn pass(&self) -> PluginPass;

    /// Name of this plugin, is used as the tab's name for custom plugins.
    fn name(&self) -> Option<String> {
        None
    }

    /// Tab to add this plugin to. Is noop for custom plugins, since they're
    /// added to their own tab.
    fn section(&self) -> Option<String> {
        None
    }

    /// Priority for adding to GUI. Is noop for custom plugins, since they're
    /// added to their own tab.
    fn priority(&self) -> Option<usize> {
        None
    }

    /// Same as `update`, but called when the app adds plugins to the GUI. Use
    /// this to add arguments and stuff.
    fn add_plugin(&self, _app: &mut StarApp, _ctx: &Context, _frame: &mut Frame) {}

    fn add_context(&self, _app: &mut StarApp, _ctx: &Context, _frame: &mut Frame) {}

    /// Called when [`StarApp`]'s `update` method is called.
    fn update(&self, _app: &mut StarApp, _ctx: &Context, _frame: &mut Frame) {}

    /// Called when [`StarApp`]'s `save` method is called.
    fn save(&self, _app: &mut StarApp, _storage: &mut dyn Storage) {}

    /// Called when [`StarApp`]'s `on_close_event` method is called.
    fn on_close_event(&self, _app: &mut StarApp) {}

    /// Called when [`StarApp`]'s `on_exit` method is called.
    fn on_exit(&self, _app: &mut StarApp) {}
}
