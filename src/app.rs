use eframe::egui;
use crate::network::{tcp::TcpTester, http::HttpTester, packet::PacketAnalyzer};

pub struct TcpIpTester {
    tcp_tester: TcpTester,
    http_tester: HttpTester,
    packet_analyzer: PacketAnalyzer,
    current_tab: Tab,
}

#[derive(PartialEq)]
enum Tab {
    Tcp,
    Http,
    Packet,
}

impl TcpIpTester {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        Self {
            tcp_tester: TcpTester::default(),
            http_tester: HttpTester::default(),
            packet_analyzer: PacketAnalyzer::default(),
            current_tab: Tab::Tcp,
        }
    }
}

impl eframe::App for TcpIpTester {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("TCP/IP Tester");
            
            ui.horizontal(|ui| {
                if ui.selectable_label(self.current_tab == Tab::Tcp, "TCP test").clicked() {
                    self.current_tab = Tab::Tcp;
                }
                if ui.selectable_label(self.current_tab == Tab::Http, "HTTP test").clicked() {
                    self.current_tab = Tab::Http;
                }
                if ui.selectable_label(self.current_tab == Tab::Packet, "Packet Analysis").clicked() {
                    self.current_tab = Tab::Packet;
                }
            });
            
            match self.current_tab {
                Tab::Tcp => { self.tcp_tester.ui(ui); },
                Tab::Http => { self.http_tester.ui(ui); },
                Tab::Packet => { self.packet_analyzer.ui(ui); },
            }
        });
    }
} 