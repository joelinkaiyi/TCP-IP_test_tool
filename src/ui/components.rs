use eframe::egui;

pub trait TabPanel {
    fn ui(&mut self, ui: &mut egui::Ui);
} 