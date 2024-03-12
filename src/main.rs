use std::net::TcpStream;
use std::time::Duration;

use anyhow::Result;
use eframe::egui;

mod commands;
mod error;

use commands::{DataType, Watch};

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
    watch_string: String,
    watch_address: u32,
    watch_datatype: DataType,
    value_string: String,
    value: u32,
}

impl Debugger {
    pub fn new(uri: &str) -> Result<Self> {
        let stream = TcpStream::connect(uri)?;
        Ok(Self {
            stream: stream,
            watches: Vec::new(),
            immediate: Watch::default(),
            watch_string: String::new(),
            watch_address: 0,
            watch_datatype: DataType::U32,
            value_string: String::new(),
            value: 0,
        })
    }

    fn watch(&mut self) {
        if let Ok(w) = commands::view_watch_values(&mut self.stream) {
            self.watches = w;
        }
    }

    // 0x00ECC430
    fn controls(&mut self, ui: &mut egui::Ui){
        // read the watches every time
        ui.with_layout(egui::Layout::left_to_right(egui::Align::TOP), |ui| {
            ui.label("Address");
            let response = ui.add(egui::TextEdit::singleline(&mut self.watch_string)
                                  .char_limit(8)
                                  .desired_width(80.0));
            if response.changed() {
                // set the watch address
                if let Ok(w) = u32::from_str_radix(&self.watch_string, 16) {
                    self.watch_address = w;
                } else {
                    // watch string is invalid, remove all non numeric characters
                    hex_only_string(&mut self.watch_string);
                }
            }
            egui::ComboBox::from_label("")
                .selected_text(format!("{}", self.watch_datatype))
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut self.watch_datatype,
                        DataType::U32, format!("{}", DataType::U32));
                    ui.selectable_value(&mut self.watch_datatype,
                        DataType::U16, format!("{}", DataType::U16));
                    ui.selectable_value(&mut self.watch_datatype,
                        DataType::U8, format!("{}", DataType::U8));
                    ui.selectable_value(&mut self.watch_datatype,
                        DataType::I32, format!("{}", DataType::I32));
                    ui.selectable_value(&mut self.watch_datatype,
                        DataType::I16, format!("{}", DataType::I16));
                    ui.selectable_value(&mut self.watch_datatype,
                        DataType::I8, format!("{}", DataType::I8));
                    ui.selectable_value(&mut self.watch_datatype,
                        DataType::F32, format!("{}", DataType::F32));
                });
            if ui.button("Watch").clicked() {
                println!("About to watch 0x{:x}", self.watch_address);
                if self.watch_address != 0 {
                    let _ = commands::watch_value(&mut self.stream, self.watch_address,
                                                  &self.watch_datatype);
                }
            }
            if ui.button("Read").clicked() {
                if let Ok(b1) = commands::read_u32(&mut self.stream, self.watch_address) {
                    self.immediate.address = self.watch_address;
                    self.immediate.nbytes = 4;
                    self.immediate.value = b1;
                }
            }
            if ui.button("Write").clicked() {
                let _ = commands::write_value(&mut self.stream, self.watch_address, self.value);
            }
            let response = ui.add(egui::TextEdit::singleline(&mut self.value_string)
                                  .char_limit(12)
                                  .desired_width(100.0));
            if response.changed() {
                // set the watch address
                if let Ok(w) = self.value_string.parse::<u32>() {
                    self.value = w;
                } else {
                    // watch string is invalid, remove all non numeric characters
                    hex_only_string(&mut self.value_string);
                }
            }
        });
    }

    fn data(&mut self, ui: &mut egui::Ui) {
        ui.with_layout(egui::Layout::top_down(egui::Align::LEFT), |ui| {
            ui.label("Watches");
            for watch in &mut self.watches {
                ui.with_layout(egui::Layout::left_to_right(egui::Align::TOP), |ui| {
                    ui.label(format!("0x{:x}", watch.address));
                    match watch.data_type {
                        DataType::U32 => ui.label(format!("{}", watch.value as u32)),
                        DataType::U16 => ui.label(format!("{}", watch.value as u16)),
                        DataType::U8 => ui.label(format!("{}", watch.value as u8)),
                        DataType::I32 => ui.label(format!("{}", watch.value as i32)),
                        DataType::I16 => ui.label(format!("{}", watch.value as i16)),
                        DataType::I8 => ui.label(format!("{}", watch.value as i8)),
                        DataType::F32 => ui.label(format!("{}", watch.value as f32)),
                    };
                    ui.label(format!("[{}]", watch.data_type));
                });
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
        self.watch();
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.with_layout(egui::Layout::top_down(egui::Align::LEFT), |ui| {
                self.controls(ui);
                self.data(ui);
            });
        });
        // refresh our watch data on a cycle
        ctx.request_repaint_after(Duration::from_millis(16));
    }
}

fn hex_only_string(s: &mut String) {
    // we only allow 32 bit addresses
    s.retain(|c| "0123456789ABCDEFabcdef".contains(c));
}
