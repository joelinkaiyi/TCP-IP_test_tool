mod app;
mod network;
mod ui;

use app::TcpIpTester;

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        initial_window_size: Some(eframe::epaint::vec2(800.0, 600.0)),
        ..Default::default()
    };
    
    eframe::run_native(
        "TCP/IP 測試工具",
        options,
        Box::new(|cc| Box::new(TcpIpTester::new(cc))),
    )
} 