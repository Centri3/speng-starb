use eframe::App;
use eframe::CreationContext;
use eframe::Frame;
use egui::CentralPanel;
use egui::Context;

#[derive(Default)]
pub struct MyEguiApp;

impl MyEguiApp {
    pub fn new(cc: &CreationContext<'_>) -> Self {
        Self::default()
    }
}

impl App for MyEguiApp {
    fn update(&mut self, ctx: &Context, frame: &mut Frame) {
        CentralPanel::default().show(ctx, |ui| {
            ui.heading("Hello World!");
        });
    }
}
