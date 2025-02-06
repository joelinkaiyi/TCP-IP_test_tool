use eframe::egui;
use std::net::SocketAddr;
use tokio::runtime::Runtime;

#[derive(Default)]
pub struct TcpTester {
    input_address: String,
    input_port: String,
    status: String,
}

impl TcpTester {
    pub fn ui(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label("IP Address:");
            ui.text_edit_singleline(&mut self.input_address);
        });
        
        ui.horizontal(|ui| {
            ui.label("Port:");
            ui.text_edit_singleline(&mut self.input_port);
        });
        
        if ui.button("Test Connection").clicked() {
            let addr = format!("{}:{}", self.input_address, self.input_port);
            if let Ok(socket_addr) = addr.parse::<SocketAddr>() {
                self.test_connection(socket_addr);
            } else {
                self.status = "Invalid address format".to_string();
            }
        }
        
        ui.label(&self.status);
    }

    fn test_connection(&mut self, addr: SocketAddr) {
        let runtime = Runtime::new().unwrap();
        runtime.block_on(async {
            match tokio::net::TcpStream::connect(addr).await {
                Ok(_) => self.status = "Connection successful".to_string(),
                Err(e) => self.status = format!("Connection failed: {}", e),
            }
        });
    }
} 