//! Port Sweeper GUI using eframe/egui.
//! Styled to match the HTML reference: radial background, window gradient, cards, and buttons.

use std::sync::Arc;
use eframe::egui::{self, pos2, Color32, Frame, Margin, Rect, RichText, Rounding, Stroke, Vec2, Visuals};
use eframe::egui::viewport::IconData;
use egui_extras::{Column, TableBuilder};
use crate::{kill_ports, list_ports, parse_port_spec, PortEntry};

// HTML: body radial gradient #24305a → #0a0f1f
const BG_TOP: Color32 = Color32::from_rgb(36, 48, 90);   // #24305a
const BG_BOTTOM: Color32 = Color32::from_rgb(10, 15, 31); // #0a0f1f

// HTML: .window linear-gradient(180deg,#1b223b,#12172b), border rgba(255,255,255,.06)
const WINDOW_BOTTOM: Color32 = Color32::from_rgb(18, 23, 43); // #12172b
const WINDOW_BORDER: Color32 = Color32::from_rgb(40, 45, 60); // ~rgba(255,255,255,.06) on dark

// HTML: .table / .card linear-gradient(180deg,#1c2240,#151a33), border rgba(255,255,255,.05)
const CARD_BG_TOP: Color32 = Color32::from_rgb(28, 34, 64);   // #1c2240
const CARD_BORDER: Color32 = Color32::from_rgb(35, 40, 55); // ~rgba(255,255,255,.05) on dark

// HTML: body #d8deff; .subtitle #8f96c5 14px
const TEXT_MAIN: Color32 = Color32::from_rgb(216, 222, 255);   // #d8deff
const SUBTITLE_COLOR: Color32 = Color32::from_rgb(143, 150, 197); // #8f96c5
const HEADER_COLOR: Color32 = Color32::from_rgb(140, 148, 200);   // #8c94c8 13px

// HTML: .status #46d08d 500; .success #58d69a; success .dot bg #58d69a, text #08120c
const STATUS_GREEN: Color32 = Color32::from_rgb(70, 208, 141);   // #46d08d
const SUCCESS_GREEN: Color32 = Color32::from_rgb(88, 214, 154);   // #58d69a
const SUCCESS_DOT_TEXT: Color32 = Color32::from_rgb(8, 18, 12);  // #08120c

// HTML: .btn linear-gradient(#c94141,#8c2323)
const RED_TOP: Color32 = Color32::from_rgb(201, 65, 65);   // #c94141
const RED_BOTTOM: Color32 = Color32::from_rgb(140, 35, 35); // #8c2323

// HTML: .refresh border rgba(255,255,255,.08), background linear-gradient(#1f2547,#181d36), color #dbe1ff
const REFRESH_BG_TOP: Color32 = Color32::from_rgb(31, 37, 71);
const REFRESH_BORDER: Color32 = Color32::from_rgb(50, 55, 72); // ~rgba(255,255,255,.08) on dark
const REFRESH_TEXT: Color32 = Color32::from_rgb(219, 225, 255);

// HTML: input background #0f142a
const INPUT_BG: Color32 = Color32::from_rgb(15, 20, 42);

/// Linear interpolation between two colors (t: 0 = top, 1 = bottom).
fn lerp_color(top: Color32, bottom: Color32, t: f32) -> Color32 {
    let r = (1.0 - t) * top.r() as f32 + t * bottom.r() as f32;
    let g = (1.0 - t) * top.g() as f32 + t * bottom.g() as f32;
    let b = (1.0 - t) * top.b() as f32 + t * bottom.b() as f32;
    Color32::from_rgb(r.round() as u8, g.round() as u8, b.round() as u8)
}

fn card_frame() -> Frame {
    Frame {
        inner_margin: Margin::symmetric(18.0, 16.0),
        outer_margin: Margin::ZERO,
        rounding: Rounding::same(10.0),
        shadow: egui::epaint::Shadow::NONE,
        fill: CARD_BG_TOP, // egui no gradient; use top color
        stroke: Stroke::new(1.0, CARD_BORDER),
    }
}

/// Load app icon from embedded logo (square, RGBA). Used for taskbar/dock/window chrome.
fn load_icon() -> Option<IconData> {
    let bytes = include_bytes!("../assets/logo.png");
    let img = image::load_from_memory(bytes).ok()?.to_rgba8();
    let (w, h) = (img.width(), img.height());
    if w == 0 || h == 0 {
        return None;
    }
    // Some platforms expect dimensions that are multiples of 4; use 256 for a standard icon size.
    let size = 256.min(w).min(h);
    let size = (size / 4) * 4;
    let size = size.max(4);
    let scaled = image::imageops::resize(
        &img,
        size,
        size,
        image::imageops::FilterType::Lanczos3,
    );
    Some(IconData {
        rgba: scaled.into_raw(),
        width: size,
        height: size,
    })
}

pub fn run() -> anyhow::Result<()> {
    let mut viewport = egui::ViewportBuilder::default()
        .with_inner_size([720.0, 600.0])
        .with_min_inner_size([520.0, 420.0]);
    if let Some(icon) = load_icon() {
        viewport = viewport.with_icon(Arc::new(icon));
    }
    let options = eframe::NativeOptions {
        viewport,
        ..Default::default()
    };
    eframe::run_native(
        "Port Sweeper",
        options,
        Box::new(|cc| {
            let mut visuals = Visuals::dark();
            visuals.panel_fill = BG_BOTTOM;
            visuals.window_fill = WINDOW_BOTTOM;
            visuals.window_rounding = Rounding::same(14.0);
            visuals.window_stroke = Stroke::new(1.0, WINDOW_BORDER);
            visuals.override_text_color = Some(TEXT_MAIN);
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
            // Smooth vertical gradient over full screen (top → bottom)
            let viewport = ui.ctx().screen_rect();
            const GRADIENT_STRIPS: u32 = 128;
            let strip_h = viewport.height() / GRADIENT_STRIPS as f32;
            for i in 0..GRADIENT_STRIPS {
                let t = (i as f32 + 0.5) / GRADIENT_STRIPS as f32;
                let y_min = viewport.min.y + i as f32 * strip_h;
                let y_max = y_min + strip_h;
                let strip_rect = Rect::from_min_max(
                    pos2(viewport.min.x, y_min),
                    pos2(viewport.max.x, y_max),
                );
                let color = lerp_color(BG_TOP, BG_BOTTOM, t);
                ui.painter().rect_filled(strip_rect, 0.0, color);
            }

            // HTML: .window padding 22px 24px 26px, border-radius 14px
            let outer = Frame {
                inner_margin: Margin { left: 24.0, right: 24.0, top: 22.0, bottom: 26.0 },
                outer_margin: Margin::ZERO,
                rounding: Rounding::same(14.0),
                shadow: egui::epaint::Shadow::NONE,
                fill: Color32::TRANSPARENT,
                stroke: Stroke::NONE,
            };
            outer.show(ui, |ui| {
                // HTML: h2 Active Ports 22px; .subtitle 14px #8f96c5
                ui.label(RichText::new("Active Ports").size(22.0).strong().color(TEXT_MAIN));
                ui.add_space(6.0);
                ui.label(
                    RichText::new("Quickly find and kill any process using an occupied port.")
                        .size(14.0)
                        .color(SUBTITLE_COLOR),
                );
                ui.add_space(18.0);

                // ── Table (HTML .table): grid 80px 1fr 90px 110px 110px, header 13px #8c94c8 ─────
                let mut kill_port: Option<u16> = None;
                card_frame().show(ui, |ui| {
                    let table = TableBuilder::new(ui)
                        .striped(false)
                        .resizable(false)
                        .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
                        .min_scrolled_height(0.0)
                        .column(Column::exact(80.0))
                        .column(Column::remainder().at_least(120.0))
                        .column(Column::exact(90.0))
                        .column(Column::exact(110.0))
                        .column(Column::exact(110.0))
                        .header(28.0, |mut h| {
                            for label in &["Port", "Process", "PID", "Status", "Action"] {
                                h.col(|ui| {
                                    ui.label(
                                        RichText::new(*label).size(13.0).color(HEADER_COLOR),
                                    );
                                });
                            }
                        });

                    table.body(|mut body| {
                        for e in &self.entries.clone() {
                            body.row(44.0, |mut row| {
                                row.col(|ui| {
                                    ui.label(RichText::new(e.port.to_string()).size(14.0).color(TEXT_MAIN));
                                });
                                row.col(|ui| {
                                    ui.label(RichText::new(&e.process_name).size(14.0).color(TEXT_MAIN));
                                });
                                row.col(|ui| {
                                    ui.label(RichText::new(e.pid.to_string()).size(14.0).color(TEXT_MAIN));
                                });
                                row.col(|ui| {
                                    ui.label(
                                        RichText::new(&e.status).size(14.0).color(STATUS_GREEN).strong(),
                                    );
                                });
                                row.col(|ui| {
                                    let btn = egui::Button::new(
                                        RichText::new("Kill").size(14.0).color(Color32::WHITE),
                                    )
                                    .fill(RED_TOP)
                                    .stroke(Stroke::new(1.0, RED_BOTTOM))
                                    .rounding(Rounding::same(8.0))
                                    .min_size(Vec2::new(90.0, 34.0));
                                    let response = ui.add(btn);
                                    if response.hovered() {
                                        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                                    }
                                    if response.clicked() {
                                        kill_port = Some(e.port);
                                    }
                                });
                            });
                        }
                    });
                });

                if let Some(port) = kill_port {
                    self.kill_port(port, now);
                }

                // HTML: .refresh margin 8px 0 20px, padding 10px 16px, border-radius 8px (text centered, shadow, scale on hover)
                ui.add_space(8.0);
                let refresh_size = Vec2::new(140.0, 36.0);
                let (rect, response) = ui.allocate_exact_size(refresh_size, egui::Sense::click());
                if response.hovered() {
                    ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                }
                if response.clicked() {
                    self.refresh();
                }
                let hover = response.hovered();
                let draw_rect = if hover { rect.expand2(Vec2::splat(2.0)) } else { rect };
                let shadow_offset = Vec2::new(2.0, 3.0);
                let shadow_rect = draw_rect.translate(shadow_offset);
                let shadow_color = Color32::from_rgba_unmultiplied(0, 0, 0, 42);
                ui.painter().rect_filled(shadow_rect, Rounding::same(8.0), shadow_color);
                ui.painter().rect_filled(draw_rect, Rounding::same(8.0), REFRESH_BG_TOP);
                ui.painter().rect_stroke(draw_rect, Rounding::same(8.0), Stroke::new(1.0, REFRESH_BORDER));
                let text = "⟳  Refresh List";
                let font = egui::FontId::proportional(14.0);
                ui.painter().text(
                    draw_rect.center(),
                    egui::Align2::CENTER_CENTER,
                    text,
                    font,
                    REFRESH_TEXT,
                );
                ui.add_space(20.0);

                // HTML: .card Kill a Port, padding 18px, border-radius 10px
                card_frame().show(ui, |ui| {
                    ui.label(
                        RichText::new("Kill a Port").size(18.0).strong().color(TEXT_MAIN),
                    );
                    ui.add_space(6.0);
                    ui.label(
                        RichText::new("Enter a port number to terminate the process.")
                            .size(14.0)
                            .color(SUBTITLE_COLOR),
                    );
                    ui.add_space(14.0);
                    ui.horizontal(|ui| {
                        let input_w = ui.available_width() - 120.0 - ui.spacing().item_spacing.x;
                        let input_frame = Frame::none()
                            .fill(INPUT_BG)
                            .rounding(Rounding::same(8.0))
                            .inner_margin(Margin::symmetric(14.0, 12.0));
                        input_frame.show(ui, |ui| {
                            // Accept only digits, commas, and hyphens (for ranges)
                            self.port_input = self
                                .port_input
                                .chars()
                                .filter(|c| c.is_ascii_digit() || *c == ',' || *c == '-')
                                .collect();
                            ui.add(
                                egui::TextEdit::singleline(&mut self.port_input)
                                    .frame(false)
                                    .hint_text("Enter port numbers (3000,8000,9000-9010)")
                                    .desired_width(input_w - 28.0)
                                    .vertical_align(egui::Align::Center),
                            );
                        });
                        let kill_btn = egui::Button::new(
                            RichText::new("Kill Port").size(14.0).color(Color32::WHITE),
                        )
                        .fill(RED_TOP)
                        .stroke(Stroke::new(1.0, RED_BOTTOM))
                        .rounding(Rounding::same(8.0))
                        .min_size(Vec2::new(100.0, 36.0));
                        let response = ui.add(kill_btn);
                        if response.hovered() {
                            ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                        }
                        if response.clicked() {
                            self.kill_from_input(now);
                        }
                    });
                });

                // HTML: .success with .dot (✓), color #58d69a, 14px
                ui.add_space(16.0);
                if !self.status_message.is_empty() {
                    if self.status_ok {
                        ui.horizontal(|ui| {
                            let dot_size = 22.0;
                            let (rect, _) = ui.allocate_exact_size(
                                Vec2::splat(dot_size),
                                egui::Sense::hover(),
                            );
                            ui.painter().circle_filled(
                                rect.center(),
                                dot_size / 2.0,
                                SUCCESS_GREEN,
                            );
                            ui.painter().text(
                                rect.center(),
                                egui::Align2::CENTER_CENTER,
                                "✓",
                                egui::FontId::proportional(14.0),
                                SUCCESS_DOT_TEXT,
                            );
                            ui.add_space(10.0);
                            ui.label(
                                RichText::new(&self.status_message)
                                    .size(14.0)
                                    .color(SUCCESS_GREEN),
                            );
                        });
                    } else {
                        ui.horizontal(|ui| {
                            ui.label(RichText::new("✗").size(14.0).color(RED_TOP));
                            ui.label(RichText::new(&self.status_message).size(14.0).color(RED_TOP));
                        });
                    }
                }
                // Extend panel to full window height so blue background fills to bottom
                ui.allocate_space(egui::vec2(0.0, ui.available_height()));
            });
        });
    }
}
