//! Port Sweeper GUI using eframe/egui.

use eframe::egui::{self, Color32, RichText, Style, Visuals};
use egui_extras::{Column, TableBuilder};
use crate::{kill_ports, list_ports, parse_port_spec, PortEntry};

pub fn run() -> anyhow::Result<()> {
    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "Port Sweeper",
        options,
        Box::new(|cc| {
            cc.egui_ctx.set_style(Style {
                visuals: Visuals::dark(),
                ..Style::default()
            });
            Ok(Box::new(PortSweeperApp::default()))
        }),
    )
    .map_err(|e| anyhow::anyhow!("{}", e))
}

#[derive(Default)]
struct PortSweeperApp {
    entries: Vec<PortEntry>,
    port_input: String,
    status_message: String,
    status_ok: bool,
    status_clear_at: Option<f64>,
}

impl PortSweeperApp {
    fn refresh(&mut self) {
        match list_ports() {
            Ok(e) => self.entries = e,
            Err(e) => {
                self.status_message = e;
                self.status_ok = false;
            }
        }
    }

    fn kill_from_input(&mut self, now: f64) {
        let spec = self.port_input.trim();
        if spec.is_empty() {
            self.status_message = "Enter port(s) to kill".to_string();
            self.status_ok = false;
            return;
        }
        let ports: Vec<u16> = match parse_port_spec(spec) {
            Ok(p) if p.is_empty() => {
                self.status_message = "No valid ports".to_string();
                self.status_ok = false;
                return;
            }
            Ok(p) => p,
            Err(e) => {
                self.status_message = e;
                self.status_ok = false;
                return;
            }
        };
        let results = kill_ports(&ports);
        let ok: Vec<_> = results.iter().filter(|r| r.success).collect();
        let fail: Vec<_> = results.iter().filter(|r| !r.success).collect();
        if fail.is_empty() {
            self.status_message = if ok.len() == 1 {
                ok[0].message.clone()
            } else {
                format!("{} port(s) terminated successfully!", ok.len())
            };
            self.status_ok = true;
            self.port_input.clear();
            self.refresh();
        } else {
            self.status_message = fail.iter().map(|r| r.message.as_str()).collect::<Vec<_>>().join("; ");
            self.status_ok = false;
        }
        self.status_clear_at = Some(now + 5.0);
    }

    fn kill_port(&mut self, port: u16, now: f64) {
        let results = kill_ports(&[port]);
        if let Some(r) = results.first() {
            self.status_message = r.message.clone();
            self.status_ok = r.success;
            if r.success {
                self.refresh();
            }
            self.status_clear_at = Some(now + 5.0);
        }
    }
}

impl eframe::App for PortSweeperApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if self.entries.is_empty() && self.status_message.is_empty() {
            self.refresh();
        }
        let now = ctx.input(|i| i.time);
        if let Some(t) = self.status_clear_at {
            if now > t {
                self.status_clear_at = None;
                self.status_message.clear();
            }
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(8.0);
                ui.heading(RichText::new("Port Sweeper").size(20.0));
                ui.add_space(16.0);
            });

            ui.add_space(8.0);
            ui.heading("Active Ports");
            ui.label("Quickly find and kill any process using an occupied port.");
            ui.add_space(8.0);

            let table = TableBuilder::new(ui)
                .striped(true)
                .resizable(true)
                .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
                .column(Column::auto().at_least(60.0))
                .column(Column::remainder().at_least(120.0))
                .column(Column::auto().at_least(60.0))
                .column(Column::auto().at_least(70.0))
                .column(Column::auto().at_least(60.0))
                .header(24.0, |mut header| {
                    header.col(|ui| {
                        ui.strong("Port");
                    });
                    header.col(|ui| {
                        ui.strong("Process");
                    });
                    header.col(|ui| {
                        ui.strong("PID");
                    });
                    header.col(|ui| {
                        ui.strong("Status");
                    });
                    header.col(|ui| {
                        ui.strong("Action");
                    });
                });

            let mut kill_port = None;
            table.body(|mut body| {
                for e in &self.entries.clone() {
                    body.row(22.0, |mut row| {
                        row.col(|ui| {
                            ui.label(e.port.to_string());
                        });
                        row.col(|ui| {
                            ui.label(&e.process_name);
                        });
                        row.col(|ui| {
                            ui.label(e.pid.to_string());
                        });
                        row.col(|ui| {
                            ui.label(RichText::new(&e.status).color(Color32::from_rgb(0x4a, 0xea, 0x6a)));
                        });
                        row.col(|ui| {
                            if ui.button(RichText::new("Kill").color(Color32::from_rgb(0xe0, 0x40, 0x40)).strong()).clicked() {
                                kill_port = Some(e.port);
                            }
                        });
                    });
                }
            });

            if let Some(port) = kill_port {
                self.kill_port(port, now);
            }

            ui.add_space(8.0);
            if ui.button("↻ Refresh List").clicked() {
                self.refresh();
            }

            ui.add_space(24.0);
            ui.heading("Kill a Port");
            ui.label("Enter a port number to terminate the process.");
            ui.add_space(8.0);
            ui.horizontal(|ui| {
                ui.add(
                    egui::TextEdit::singleline(&mut self.port_input)
                        .hint_text("Enter port numbers (3000,8000,9000-9010)")
                        .desired_width(320.0),
                );
                if ui.button(RichText::new("Kill Port").color(Color32::from_rgb(0xe0, 0x40, 0x40)).strong()).clicked() {
                    self.kill_from_input(now);
                }
            });

            ui.add_space(16.0);
            if !self.status_message.is_empty() {
                if self.status_ok {
                    ui.horizontal(|ui| {
                        ui.label(RichText::new("✓").color(Color32::from_rgb(0x4a, 0xea, 0x6a)).size(16.0));
                        ui.label(&self.status_message);
                    });
                } else {
                    ui.horizontal(|ui| {
                        ui.label(RichText::new("✗").color(Color32::from_rgb(0xe0, 0x40, 0x40)).size(16.0));
                        ui.label(RichText::new(&self.status_message).color(Color32::from_rgb(0xe0, 0x40, 0x40)));
                    });
                }
            }
        });
    }
}
