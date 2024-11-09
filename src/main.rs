use config::AppConfig;
use egui::RichText;
use egui::{Color32, Vec2, WidgetText};
use egui_overlay::egui_render_three_d::ThreeDBackend as DefaultGfxBackend;
use egui_overlay::EguiOverlay;
use egui_window_glfw_passthrough::glfw::{Action, Key, WindowEvent};
use hyprland::dispatch;
use hyprland::dispatch::Dispatch;
use hyprland::dispatch::{DispatchType, DispatchType::*};
use hyprland::{
    data::{Client, Monitor},
    dispatch::WindowIdentifier,
    keyword::{Keyword, OptionValue},
    shared::{HyprDataActive, HyprDataActiveOptional},
};

mod config;

pub struct AppState {
    pub frame: u64,
    pub config: config::AppConfig,
    pub clicks: Vec<(usize, usize)>,
    pub target_client: Client,
    pub monitor: Monitor,
}


fn main() {
    use tracing_subscriber::{fmt, prelude::*, EnvFilter};
    // if RUST_LOG is not set, we will use the following filters
    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(
            EnvFilter::try_from_default_env()
                .unwrap_or(EnvFilter::new("warn,wgpu=warn,naga=warn")),
        )
        .init();
    let current = Client::get_active()
        .expect("Failed to get the target client window!")
        .expect("No current valid client window.");
    let monitor = Monitor::get_active().expect("Failed to get the active monitor");

    let border_width: u16 = match Keyword::get("general:border_size")
        .expect("Failed to get hyprland border settings")
        .value
    {
        OptionValue::Int(border) => border as u16,
        OptionValue::Float(border) => border as u16,
        _ => 5,
    };

    let appconfig = AppConfig::load().expect("Failed to load/generate config.");

    let app_state = AppState {
        frame: 0,
        config: appconfig,
        clicks: Vec::new(),
        target_client: current,
        monitor,
    };
    egui_overlay::start(app_state);
    
}

const ICON_BYTES: &[u8] = include_bytes!("../resources/hypr-gridtile.png");



fn calc_rowcol_bounds(clicks: &Vec<(usize, usize)>) -> (usize, usize, usize, usize) {
    if clicks.is_empty() {
        (999, 999, 999, 999)
    } else if clicks.len() == 1 {
        (clicks[0].0, clicks[0].1, clicks[0].0, clicks[0].1)
    } else {
        (
            clicks[0].0.min(clicks[1].0),
            clicks[0].1.min(clicks[1].1),
            clicks[1].0.max(clicks[0].0),
            clicks[1].1.max(clicks[0].1),
        )
    }
}

impl EguiOverlay for AppState {
    fn gui_run(
        &mut self,
        egui_context: &egui::Context,
        _default_gfx_backend: &mut DefaultGfxBackend, //DefaultGfxBackend,
        glfw_backend: &mut egui_window_glfw_passthrough::GlfwBackend,
    ) {
        // egui_context.all_styles_mut(|style: &mut Style| {
        // let text_styles: BTreeMap<TextStyle, FontId> = [
        //     (TextStyle::Button, FontId::new(25.0, Proportional)),
        //     (TextStyle::Heading, FontId::new(25.0, Proportional)),
        //     (TextStyle::Body, FontId::new(20.0, Proportional)),
        //     (TextStyle::Monospace, FontId::new(12.0, Monospace)),
        //     (TextStyle::Small, FontId::new(8.0, Proportional)),
        // ]
        // .into();
        // style.text_styles = text_styles.clone();
        // style.spacing.slider_rail_height=16.;
        // style.text_styles.insert(TextStyle::Button, FontId::new(16.0, Proportional));
        // });
        let evs: Vec<WindowEvent> = glfw_backend.frame_events.clone();
        if !evs.is_empty() {
            for ev in evs {
                if let WindowEvent::Key(key, _code, Action::Press, _) = ev {
                    if key == Key::Escape {
                        glfw_backend.window.set_should_close(true);
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
        }

        // first frame logic
        if self.frame == 0 {
            glfw_backend.set_title("Hypr-GridTile".to_string());
            let icon = image::load_from_memory(ICON_BYTES).unwrap().to_rgba8();
            {
                let pixels = icon
                    .pixels()
                    .map(|pixel| u32::from_le_bytes(pixel.0))
                    .collect();
                let icon = egui_window_glfw_passthrough::glfw::PixelImage {
                    width: icon.width(),
                    height: icon.height(),
                    pixels,
                };
                glfw_backend.window.set_icon_from_pixels(vec![icon]);
            }
        }
        let fbsize = glfw_backend.window.get_framebuffer_size();
        // just some controls to show how you can use glfw_backend
        egui::Window::new("Foo")
            .collapsible(false)
            .title_bar(false)
            .resizable([true, true])
            .min_size(Vec2::new(fbsize.0 as f32, fbsize.1 as f32))
            // .frame(egui::Frame::default().)
            .frame(egui::Frame {
                fill: egui::Color32::TRANSPARENT,
                inner_margin: egui::Margin::same(16.0),
                outer_margin: egui::Margin::same(16.0),
                ..Default::default()
            })
            .show(egui_context, |ui| {
                self.frame += 1;

                ui.horizontal(|ui| {
                    ui.label("Columns:");
                    ui.add(egui::Slider::new(&mut self.config.columns, 1..=9u16));
                    ui.label("Rows:");
                    ui.add(egui::Slider::new(&mut self.config.rows, 1..=3u16));
                });

                let (x0, y0, x1, y1) = calc_rowcol_bounds(&self.clicks);

                let button_width = (fbsize.0 as f32 - 32.) / self.config.columns as f32;
                let button_height = (fbsize.1 as f32 - 64.) / self.config.rows as f32;

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
                        }
                    });
                }

                ui.allocate_space(ui.available_size());

                if self.clicks.len() >= 2 {
                    let (x0, y0, x1, y1) = calc_rowcol_bounds(&self.clicks);
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
                    let window_id = WindowIdentifier::Address(self.target_client.address.clone());
                    if !self.target_client.floating {
                        dispatch!(ToggleFloating, Some(window_id.clone()))
                            .expect("This better be floating now eh?!");
                    }

                    Dispatch::call(ResizeWindowPixel(
                        dispatch::Position::Exact(
                            new_width.try_into().unwrap(),
                            new_height.try_into().unwrap(),
                        ),
                        window_id.clone(),
                    ))
                    .expect("Should be resized naow.");
                    Dispatch::call(ResizeWindowPixel(
                        dispatch::Position::Exact(
                            new_width.try_into().unwrap(),
                            new_height.try_into().unwrap(),
                        ),
                        window_id.clone(),
                    ))
                    .expect("Should be resized naow.");
                    Dispatch::call(MoveWindowPixel(
                        dispatch::Position::Exact(
                            left_ofs.try_into().unwrap(),
                            top_ofs.try_into().unwrap(),
                        ),
                        window_id.clone(),
                    ))
                    .expect("Should be resized naow.");
                    Dispatch::call(MoveWindowPixel(
                        dispatch::Position::Exact(
                            left_ofs.try_into().unwrap(),
                            top_ofs.try_into().unwrap(),
                        ),
                        window_id.clone(),
                    ))
                    .expect("Should be resized naow.");

                    self.config.save();
                    glfw_backend.window.set_should_close(true)
                }
            });
        egui_context.request_repaint();
    }
}
