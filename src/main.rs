use std::thread;
use std::time::Duration;

use config::AppConfig;
use egui::RichText;
use egui::{Align2, Color32, Vec2, WidgetText};
use egui_overlay::egui_render_three_d::ThreeDBackend as DefaultGfxBackend;
use egui_overlay::EguiOverlay;
use egui_window_glfw_passthrough::glfw::{Action, Key, WindowEvent};
use hyprland::{
    data::{Client, Monitor},
    shared::{HyprDataActive, HyprDataActiveOptional},
};

mod config;
mod icon;
mod util;

const NOT_CLOSING: u64 = u64::MAX;

pub struct AppState {
    pub frame: u64,
    pub config: config::AppConfig,
    pub clicks: Vec<(usize, usize)>,
    pub target_client: Client,
    pub self_client: Option<Client>,
    pub monitor: Monitor,
    pub close_at: u64,
    pub do_tile: bool,
}

fn main() {
    util::init_logging();
    let current = Client::get_active()
        .expect("Failed to get the target client window!")
        .expect("No current valid client window.");
    let monitor = Monitor::get_active().expect("Failed to get the active monitor");

    let appconfig = AppConfig::load().expect("Failed to load/generate config.");

    let app_state = AppState {
        frame: 0,
        config: appconfig,
        clicks: Vec::new(),
        target_client: current,
        self_client: None,
        monitor,
        close_at: NOT_CLOSING,
        do_tile: false,
    };
    egui_overlay::start(app_state);
}

impl EguiOverlay for AppState {
    fn gui_run(
        &mut self,
        egui_context: &egui::Context,
        _default_gfx_backend: &mut DefaultGfxBackend, //DefaultGfxBackend,
        glfw_backend: &mut egui_window_glfw_passthrough::GlfwBackend,
    ) {
        // Handle all key events we require...
        if !glfw_backend.window.is_focused() {
            glfw_backend.window.focus();
        }
        let evs: Vec<WindowEvent> = glfw_backend.frame_events.clone();
        if !evs.is_empty() {
            for ev in evs {
                if let WindowEvent::Key(key, _code, Action::Press, _) = ev {
                    if key == Key::Escape {
                        glfw_backend.window.set_should_close(true);
                        self.config.save().expect("Failed to save config!");
                    }
                }
                if let WindowEvent::Key(key, _code, Action::Release, _) = ev {
                    if let Some(name) = key.get_name() {
                        for row in 0..self.config.rows as usize {
                            for col in 0..self.config.columns as usize {
                                if self.config.keeb[row][col].to_uppercase() == name.to_uppercase()
                                {
                                    self.clicks.push((col, row))
                                }
                            }
                        }
                    }
                }
            }
        } //WTAF that's a lot of nesting. Too much lisp lately, friend!

        // first frame logic
        let fbsize = glfw_backend.window.get_framebuffer_size();
        if self.frame == 0 {
            glfw_backend.set_title("Hypr-GridTile".to_string());
            icon::glfw_set_icon(glfw_backend);
            // println!("fbsize: {:?}", fbsize);
        } else if self.frame == 1 {
            if let Some(client) = Client::get_active().unwrap_or(None) {
                self.self_client = Some(client.clone());
                if self.config.fullscreen_at_start {
                    util::force_fullscreen_window(&client);
                }
            }
        }
        egui_context.all_styles_mut(|style| {
            style.visuals.window_fill = Color32::from_black_alpha(120);
            style.visuals.dark_mode = true;
        });

        // just some controls to show how you can use glfw_backend
        egui::Window::new("Foo")
            .collapsible(false)
            .title_bar(false)
            .anchor(Align2::LEFT_TOP, Vec2::new(0., 0.))
            .resizable([false, false])
            .min_size(Vec2::new(fbsize.0 as f32 / 0.72, fbsize.1 as f32 / 0.72))
            .fixed_size(Vec2::new(fbsize.0 as f32 + 32., fbsize.1 as f32 + 32.))
            .frame(egui::Frame {
                inner_margin: egui::Margin::same(16.0),
                outer_margin: egui::Margin::same(8.0),
                fill: Color32::from_black_alpha(20),
                ..Default::default()
            })
            .show(egui_context, |ui| {
                self.frame += 1;

                ui.vertical(|ui| {
                    egui::Grid::new("Config")
                        .with_row_color(|_, _| Some(Color32::from_black_alpha(220)))
                        // .num_columns(4)
                        .spacing(Vec2::new(16., 0.))
                        .show(ui, |ui| {
                            ui.allocate_space(Vec2::new(0., 0.));
                            ui.end_row();
                            ui.label("Columns:");
                            ui.add(egui::Slider::new(&mut self.config.columns, 1..=9u16));
                            ui.label("Rows:");
                            ui.add(egui::Slider::new(&mut self.config.rows, 1..=3u16));
                            ui.checkbox(
                                &mut self.config.fullscreen_at_start,
                                "Fullscreen at start?",
                            );
                            ui.end_row();
                            ui.allocate_space(Vec2::new(0., 0.));
                            ui.end_row();
                        });
                    ui.add_space(5.);

                    let button_width = (fbsize.0 as f32 - 32.) / self.config.columns as f32;
                    let button_height = (fbsize.1 as f32 - 64. - 16.) / self.config.rows as f32;

                    let (x0, y0, x1, y1) = util::calc_rowcol_bounds(&self.clicks);

                    for row in 0..self.config.rows as usize {
                        ui.horizontal(|ui| {
                            for col in 0..self.config.columns as usize {
                                let button = egui::Button::new(WidgetText::RichText(
                                    RichText::new(self.config.keeb[row][col].clone()).size(32.),
                                ))
                                .min_size(Vec2::new(button_width - 16., button_height - 16.));

                                let button = if col >= x0 && col <= x1 && row >= y0 && row <= y1 {
                                    button.fill(Color32::from_rgb(255, 255, 255))
                                } else {
                                    button
                                };

                                if ui.add(button).clicked() {
                                    self.clicks.push((col, row));
                                };
                                ui.add_space(8.);
                            }
                        });
                        ui.add_space(8.);
                    }

                    // Fill the damn thing up.
                    // ui.allocate_space(ui.available_size());
                });

                if self.do_tile {
                    let (x0, y0, x1, y1) = util::calc_rowcol_bounds(&self.clicks);
                    // Set the window position and bail out.
                    let gaps_in = self.config.margin;
                    let gaps_out = self.config.margin;
                    let col_width = (self.monitor.width - 2 * gaps_out) / self.config.columns;
                    let row_height = (self.monitor.height
                        - (gaps_out * 2)
                        - self.config.waybar_height
                        - self.config.border_width)
                        / self.config.rows;
                    let mut new_width = (((x1 + 1) - x0) as u16 * col_width) - gaps_in;
                    let mut new_height = ((y1 + 1) - y0) as u16 * row_height - gaps_in;
                    let mut left_ofs = self.monitor.x as u16 + gaps_out + (x0 as u16 * col_width);
                    let mut top_ofs = self.monitor.y as u16
                        + gaps_out
                        + self.config.waybar_height
                        + (y0 as u16 * row_height);
                    if x0 > 0 {
                        left_ofs += gaps_in;
                        new_width -= gaps_in;
                    }
                    if y0 > 0 {
                        top_ofs += gaps_in;
                        new_height -= gaps_in;
                    }
                    if x1 as u16 == self.config.columns {
                        new_width -= gaps_out;
                    }
                    if y1 as u16 == self.config.rows {
                        new_height -= gaps_out;
                    }

                    // Force the current client to float.
                    util::force_float_window(&self.target_client);
                    for _i in 0..10 {
                        // println!("Resizing and moving...");
                        util::move_and_resize_hypr_win(
                            &self.target_client,
                            left_ofs,
                            top_ofs,
                            new_width,
                            new_height,
                        );
                        if self.target_client.size == (new_width as i16, new_height as i16)
                            && self.target_client.at == (left_ofs as i16, top_ofs as i16)
                        {
                            break;
                        };
                        thread::sleep(Duration::from_millis(50));
                        if let Some(client) = self.self_client.clone() {
                            // println!("Focusing myself");
                            util::force_focus_window(&client);
                        }
                    }

                    // Ensure a redraw with the tile position before we do the
                    // move and resize stuff.

                    self.close_at = self.frame + 20;
                    self.config.save().expect("Failed to save config!");
                    // glfw_backend.window.focus();
                    if let Some(client) = self.self_client.clone() {
                        // println!("Focusing myself");
                        util::force_focus_window(&client);
                    }
                    // else{
                    //     println!("No current self window to focus.");
                    // }

                    self.do_tile = false;
                }
                if self.clicks.len() >= 2 && self.close_at == NOT_CLOSING {
                    self.do_tile = true;
                }
                if self.close_at <= self.frame {
                    glfw_backend.window.set_should_close(true);
                    self.close_at = NOT_CLOSING;
                }
            });
        egui_context.request_repaint();
    }
}
