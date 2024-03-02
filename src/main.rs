use std::net::TcpStream;

use anyhow::Result;
use eframe::egui;

mod commands;
mod error;

use commands::Watch;

fn main() -> Result<()> {
    let options = eframe::NativeOptions::default();
    let _ = eframe::run_native("Memory Debugger", options,
        Box::new(|_| Box::new(Debugger::new("127.0.0.1:3333")
                              .expect("Cannot load UI")))

    );
    Ok(())
}

pub struct Debugger {
    stream: TcpStream,
    watches: Vec<Watch>,
    immediate: Watch,
}

impl Debugger {
    pub fn new(uri: &str) -> Result<Self> {
        let stream = TcpStream::connect(uri)?;
        Ok(Self {
            stream: stream,
            watches: Vec::new(),
            immediate: Watch::default(),
        })
    }

    fn controls(&mut self, ui: &mut egui::Ui){
        ui.with_layout(egui::Layout::left_to_right(egui::Align::TOP), |ui| {
            if ui.button("Watch").clicked() {
                if let Ok(_) = commands::watch_value::<u32, 4>(&mut self.stream, 0x00ECC430) {
                    if let Ok(w) = commands::view_watch_values(&mut self.stream) {
                        self.watches = w;
                    }
                }
            }
            if ui.button("Read").clicked() {
                if let Ok(b1) = commands::read_u32(&mut self.stream, 0x00ECC430) {
                    self.immediate.address = 0x000ECC430;
                    self.immediate.nbytes = 4;
                    self.immediate.value = b1;
                }
            }
            if ui.button("Write").clicked() {
                let _ = commands::write_value(&mut self.stream, 0x00ECC430, 2_u32);
            }
        });
    }

    fn data(&mut self, ui: &mut egui::Ui) {
        ui.with_layout(egui::Layout::top_down(egui::Align::LEFT), |ui| {
            ui.label("Watches");
            for watch in &self.watches {
                ui.label(format!("0x{:x} = {}", watch.address, watch.value));
            }
            ui.label("Immediate");
            if self.immediate.address != 0 {
                ui.label(format!("0x{:x} = {}", self.immediate.address,
                                 self.immediate.value));
            }
        });
    }
}

impl eframe::App for Debugger {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.with_layout(egui::Layout::top_down(egui::Align::LEFT), |ui| {
                self.controls(ui);
                self.data(ui);
            });
        });
    }
}
