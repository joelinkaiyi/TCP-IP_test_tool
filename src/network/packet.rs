use eframe::egui;
use pnet::datalink::{self, NetworkInterface};
use pnet::packet::ethernet::{EtherTypes, EthernetPacket};
use pnet::packet::ip::IpNextHeaderProtocols;
use pnet::packet::ipv4::Ipv4Packet;
use pnet::packet::tcp::TcpPacket;
use pnet::packet::udp::UdpPacket;
use pnet::packet::Packet;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread;
use std::time::{Duration, SystemTime};

#[derive(Clone)]
pub struct PacketInfo {
    timestamp: SystemTime,
    source: String,
    destination: String,
    protocol: String,
    length: usize,
    details: String,
}

pub struct PacketAnalyzer {
    selected_interface: Option<NetworkInterface>,
    interfaces: Vec<NetworkInterface>,
    captured_packets: Vec<PacketInfo>,
    is_capturing: bool,
    packet_receiver: Option<Receiver<PacketInfo>>,
    capture_sender: Option<Sender<bool>>,
    filter_text: String,
}

impl Default for PacketAnalyzer {
    fn default() -> Self {
        Self {
            selected_interface: None,
            interfaces: datalink::interfaces(),
            captured_packets: Vec::new(),
            is_capturing: false,
            packet_receiver: None,
            capture_sender: None,
            filter_text: String::new(),
        }
    }
}

impl PacketAnalyzer {
    pub fn ui(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label("Network Interface:");
            egui::ComboBox::from_label("")
                .selected_text(
                    self.selected_interface
                        .as_ref()
                        .map_or("Choose Interface".to_string(), |i| i.name.clone()),
                )
                .show_ui(ui, |ui| {
                    for iface in &self.interfaces {
                        ui.selectable_value(
                            &mut self.selected_interface,
                            Some(iface.clone()),
                            &iface.name,
                        );
                    }
                });

            if !self.is_capturing {
                if ui.button("Start Capture").clicked() && self.selected_interface.is_some() {
                    self.start_capture();
                }
            } else {
                if ui.button("Stop Capture").clicked() {
                    self.stop_capture();
                }
            }
        });

        ui.horizontal(|ui| {
            ui.label("Filter:");
            ui.text_edit_singleline(&mut self.filter_text);
        });

        // 檢查是否有新的封包
        if let Some(receiver) = &self.packet_receiver {
            while let Ok(packet_info) = receiver.try_recv() {
                self.captured_packets.push(packet_info);
            }
        }

        // 顯示封包列表
        egui::ScrollArea::vertical().show(ui, |ui| {
            ui.spacing_mut().item_spacing.y = 2.0;

            for packet in self.captured_packets.iter().rev() {
                if self.filter_text.is_empty()
                    || packet.source.contains(&self.filter_text)
                    || packet.destination.contains(&self.filter_text)
                    || packet.protocol.contains(&self.filter_text)
                {
                    ui.group(|ui| {
                        ui.horizontal(|ui| {
                            ui.label(format!(
                                "Time: {:?}",
                                packet
                                    .timestamp
                                    .duration_since(SystemTime::UNIX_EPOCH)
                                    .unwrap_or(Duration::from_secs(0))
                                    .as_secs()
                            ));
                            ui.label(format!("Source: {}", packet.source));
                            ui.label(format!("Destination: {}", packet.destination));
                            ui.label(format!("Protocol: {}", packet.protocol));
                            ui.label(format!("Length: {} bytes", packet.length));
                        });
                        if !packet.details.is_empty() {
                            ui.label(&packet.details);
                        }
                    });
                }
            }
        });
    }

    fn start_capture(&mut self) {
        if let Some(interface) = self.selected_interface.clone() {
            let (packet_sender, packet_receiver) = channel();
            let (capture_sender, capture_receiver) = channel();

            self.packet_receiver = Some(packet_receiver);
            self.capture_sender = Some(capture_sender);
            self.is_capturing = true;

            thread::spawn(
                move || match datalink::channel(&interface, Default::default()) {
                    Ok(datalink::Channel::Ethernet(_, mut rx)) => {
                        while let Ok(packet) = rx.next() {
                            if capture_receiver.try_recv().is_ok() {
                                break;
                            }

                            if let Some(ethernet) = EthernetPacket::new(packet) {
                                let mut packet_info = PacketInfo {
                                    timestamp: SystemTime::now(),
                                    source: ethernet.get_source().to_string(),
                                    destination: ethernet.get_destination().to_string(),
                                    protocol: "Ethernet".to_string(),
                                    length: ethernet.packet().len(),
                                    details: String::new(),
                                };

                                match ethernet.get_ethertype() {
                                    EtherTypes::Ipv4 => {
                                        if let Some(ipv4) = Ipv4Packet::new(ethernet.payload()) {
                                            packet_info.source = ipv4.get_source().to_string();
                                            packet_info.destination =
                                                ipv4.get_destination().to_string();

                                            match ipv4.get_next_level_protocol() {
                                                IpNextHeaderProtocols::Tcp => {
                                                    if let Some(tcp) =
                                                        TcpPacket::new(ipv4.payload())
                                                    {
                                                        packet_info.protocol = "TCP".to_string();
                                                        packet_info.details = format!(
                                                            "源端口: {}, 目標端口: {}, Flags: {}",
                                                            tcp.get_source(),
                                                            tcp.get_destination(),
                                                            tcp.get_flags()
                                                        );
                                                    }
                                                }
                                                IpNextHeaderProtocols::Udp => {
                                                    if let Some(udp) =
                                                        UdpPacket::new(ipv4.payload())
                                                    {
                                                        packet_info.protocol = "UDP".to_string();
                                                        packet_info.details = format!(
                                                            "源端口: {}, 目標端口: {}",
                                                            udp.get_source(),
                                                            udp.get_destination()
                                                        );
                                                    }
                                                }
                                                _ => {
                                                    packet_info.protocol = "IPv4".to_string();
                                                }
                                            }
                                        }
                                    }
                                    _ => {}
                                }

                                let _ = packet_sender.send(packet_info);
                            }
                        }
                    }
                    _ => {
                        println!("不支援的網路介面類型");
                    }
                },
            );
        }
    }

    fn stop_capture(&mut self) {
        if let Some(sender) = self.capture_sender.take() {
            let _ = sender.send(true);
        }
        self.is_capturing = false;
    }
}
