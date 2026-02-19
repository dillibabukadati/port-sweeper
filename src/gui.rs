//! Port Sweeper GUI using eframe/egui.

use eframe::egui::{self, Color32, Frame, Margin, RichText, Rounding, Stroke, Vec2, Visuals};
use egui_extras::{Column, TableBuilder};
use crate::{kill_ports, list_ports, parse_port_spec, PortEntry};

const BG_COLOR:      Color32 = Color32::from_rgb(0x16, 0x16, 0x22);
const CARD_BG:       Color32 = Color32::from_rgb(0x20, 0x20, 0x2c);
const CARD_BORDER:   Color32 = Color32::from_rgb(0x38, 0x38, 0x50);
const RED:           Color32 = Color32::from_rgb(0xcc, 0x22, 0x22);
const HEADING_WHITE: Color32 = Color32::from_rgb(0xf2, 0xf2, 0xf2);
const MUTED:         Color32 = Color32::from_rgb(0x88, 0x88, 0x9e);
const HEADER_GRAY:   Color32 = Color32::from_rgb(0x78, 0x78, 0x90);
const GREEN:         Color32 = Color32::from_rgb(0x4a, 0xd4, 0x6a);
const SUCCESS_BG:    Color32 = Color32::from_rgb(0x18, 0x32, 0x1e);

fn card_frame() -> Frame {
    Frame {
        inner_margin: Margin::same(16.0),
        outer_margin: Margin::ZERO,
        rounding: Rounding::same(8.0),
        shadow: egui::epaint::Shadow::NONE,
        fill: CARD_BG,
        stroke: Stroke::new(1.0, CARD_BORDER),
    }
}

pub fn run() -> anyhow::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([720.0, 600.0])
            .with_min_inner_size([520.0, 420.0]),
        ..Default::default()
    };
    eframe::run_native(
        "Port Sweeper",
        options,
        Box::new(|cc| {
            let mut visuals = Visuals::dark();
            visuals.panel_fill = BG_COLOR;
            visuals.window_fill = BG_COLOR;
            visuals.extreme_bg_color = Color32::from_rgb(0x12, 0x12, 0x1c);
            visuals.widgets.noninteractive.bg_fill = CARD_BG;
            visuals.widgets.inactive.bg_fill = Color32::from_rgb(0x2a, 0x2a, 0x38);
            visuals.widgets.hovered.bg_fill = Color32::from_rgb(0x30, 0x30, 0x42);
            visuals.widgets.active.bg_fill = Color32::from_rgb(0x28, 0x28, 0x38);
            visuals.override_text_color = Some(HEADING_WHITE);
            cc.egui_ctx.set_visuals(visuals);
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
            self.status_message = fail
                .iter()
                .map(|r| r.message.as_str())
                .collect::<Vec<_>>()
                .join("; ");
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
            // Outer padding
            let outer = Frame {
                inner_margin: Margin { left: 24.0, right: 24.0, top: 20.0, bottom: 20.0 },
                outer_margin: Margin::ZERO,
                rounding: Rounding::ZERO,
                shadow: egui::epaint::Shadow::NONE,
                fill: Color32::TRANSPARENT,
                stroke: Stroke::NONE,
            };
            outer.show(ui, |ui| {
                // ── Active Ports heading ────────────────────────────────────
                ui.label(RichText::new("Active Ports").size(22.0).strong().color(HEADING_WHITE));
                ui.add_space(4.0);
                // Inline-bold "kill" approximation: separate labels
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 3.0;
                    ui.label(RichText::new("Quickly find and").size(13.0).color(MUTED));
                    ui.label(RichText::new("kill").size(13.0).strong().color(MUTED));
                    ui.label(RichText::new("any process using an occupied port.").size(13.0).color(MUTED));
                });
                ui.add_space(14.0);

                // ── Table card ──────────────────────────────────────────────
                let mut kill_port: Option<u16> = None;
                card_frame().show(ui, |ui| {
                    let available_w = ui.available_width();
                    let table = TableBuilder::new(ui)
                        .striped(false)
                        .resizable(false)
                        .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
                        .min_scrolled_height(0.0)
                        .column(Column::exact(80.0))
                        .column(Column::remainder().at_least(120.0))
                        .column(Column::exact(80.0))
                        .column(Column::exact(90.0))
                        .column(Column::exact(100.0))
                        .header(28.0, |mut h| {
                            for label in &["Port", "Process", "PID", "Status", "Action"] {
                                h.col(|ui| {
                                    ui.label(
                                        RichText::new(*label).size(12.0).color(HEADER_GRAY),
                                    );
                                });
                            }
                        });

                    table.body(|mut body| {
                        for e in &self.entries.clone() {
                            body.row(40.0, |mut row| {
                                row.col(|ui| {
                                    ui.label(RichText::new(e.port.to_string()).size(13.5));
                                });
                                row.col(|ui| {
                                    ui.label(RichText::new(&e.process_name).size(13.5));
                                });
                                row.col(|ui| {
                                    ui.label(RichText::new(e.pid.to_string()).size(13.5));
                                });
                                row.col(|ui| {
                                    ui.label(
                                        RichText::new(&e.status).size(13.5).color(GREEN),
                                    );
                                });
                                row.col(|ui| {
                                    let btn = egui::Button::new(
                                        RichText::new("Kill")
                                            .size(13.0)
                                            .color(Color32::WHITE)
                                            .strong(),
                                    )
                                    .fill(RED)
                                    .rounding(Rounding::same(6.0))
                                    .min_size(Vec2::new(80.0, 30.0));
                                    if ui.add(btn).clicked() {
                                        kill_port = Some(e.port);
                                    }
                                });
                            });
                        }
                    });

                    // suppress unused-variable warning for available_w
                    let _ = available_w;
                });

                if let Some(port) = kill_port {
                    self.kill_port(port, now);
                }

                // ── Refresh List ────────────────────────────────────────────
                ui.add_space(12.0);
                let refresh_btn = egui::Button::new(
                    RichText::new("⟳  Refresh List").size(13.0).color(HEADING_WHITE),
                )
                .fill(Color32::from_rgb(0x2c, 0x2c, 0x3c))
                .stroke(Stroke::new(1.0, CARD_BORDER))
                .rounding(Rounding::same(6.0))
                .min_size(Vec2::new(130.0, 30.0));
                if ui.add(refresh_btn).clicked() {
                    self.refresh();
                }

                // ── Kill a Port card ────────────────────────────────────────
                ui.add_space(20.0);
                card_frame().show(ui, |ui| {
                    ui.label(
                        RichText::new("Kill a Port").size(17.0).strong().color(HEADING_WHITE),
                    );
                    ui.add_space(4.0);
                    ui.label(
                        RichText::new("Enter a port number to terminate the process.")
                            .size(12.0)
                            .color(MUTED),
                    );
                    ui.add_space(12.0);
                    ui.horizontal(|ui| {
                        let input_w = ui.available_width() - 130.0 - ui.spacing().item_spacing.x;
                        ui.add(
                            egui::TextEdit::singleline(&mut self.port_input)
                                .hint_text("Enter port numbers (3000,8000,9000-9010)")
                                .desired_width(input_w)
                                .vertical_align(egui::Align::Center),
                        );
                        let kill_btn = egui::Button::new(
                            RichText::new("Kill Port")
                                .size(13.5)
                                .color(Color32::WHITE)
                                .strong(),
                        )
                        .fill(RED)
                        .rounding(Rounding::same(6.0))
                        .min_size(Vec2::new(110.0, 34.0));
                        if ui.add(kill_btn).clicked() {
                            self.kill_from_input(now);
                        }
                    });
                });

                // ── Status message ──────────────────────────────────────────
                ui.add_space(14.0);
                if !self.status_message.is_empty() {
                    if self.status_ok {
                        Frame {
                            inner_margin: Margin::symmetric(14.0, 10.0),
                            outer_margin: Margin::ZERO,
                            rounding: Rounding::same(8.0),
                            shadow: egui::epaint::Shadow::NONE,
                            fill: SUCCESS_BG,
                            stroke: Stroke::new(1.0, Color32::from_rgb(0x28, 0x72, 0x3a)),
                        }
                        .show(ui, |ui| {
                            ui.horizontal(|ui| {
                                ui.label(
                                    RichText::new("✓").size(15.0).color(GREEN).strong(),
                                );
                                ui.add_space(6.0);
                                ui.label(
                                    RichText::new(&self.status_message)
                                        .size(13.0)
                                        .color(GREEN),
                                );
                            });
                        });
                    } else {
                        ui.horizontal(|ui| {
                            ui.label(RichText::new("✗").size(15.0).color(RED));
                            ui.label(RichText::new(&self.status_message).size(13.0).color(RED));
                        });
                    }
                }
            });
        });
    }
}
