use egui_window_glfw_passthrough::glfw::{Action, WindowEvent};
use hyprland::{
    data::{Animations, Binds, Client, Clients, Monitor, Monitors, Workspace, Workspaces},
    dispatch::WindowIdentifier,
    keyword::{Keyword, OptionValue},
    shared::{HyprData, HyprDataActive, HyprDataActiveOptional},
};

use egui::epaint::WHITE_UV;
use egui::{Color32, Vec2};
use egui_overlay::EguiOverlay;
use hyprland::dispatch;
use hyprland::dispatch::DispatchType::*;
use hyprland::dispatch::{
    Corner, Dispatch, DispatchType, FullscreenType, WorkspaceIdentifierWithSpecial,
};
// #[cfg(feature = "three_d")]
use egui_overlay::egui_render_three_d::ThreeDBackend as DefaultGfxBackend;
use winit::event::KeyEvent;
// #[cfg(feature = "wgpu")]
// use egui_render_wgpu::WgpuBackend as DefaultGfxBackend;
// #[cfg(not(any(feature = "three_d", feature = "wgpu")))]
// compile_error!("you must enable either `three_d` or `wgpu` feature to run this example");
fn main() {
    use tracing_subscriber::{fmt, prelude::*, EnvFilter};
    // if RUST_LOG is not set, we will use the following filters
    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(
            EnvFilter::try_from_default_env()
                .unwrap_or(EnvFilter::new("debug,wgpu=warn,naga=warn")),
        )
        .init();
    let keeb = Vec::from([
        "QWERTYUIOP".chars().map(|c| c.to_string()).collect(),
        "ASDFGHJKL;".chars().map(|c| c.to_string()).collect(),
        "ZXCVBNM,.".chars().map(|c| c.to_string()).collect(),
    ]);
    let current = Client::get_active()
        .expect("Failed to get the target client window!")
        .expect("No current valid client window.");
    let monitor = Monitor::get_active().expect("Failed to get the active monitor");
    let margin = 15;
    let waybar_height = 48;

    let border_width: u16 = match Keyword::get("general:border_size")
        .expect("Failed to get hyprland border settings")
        .value
    {
        OptionValue::Int(border) => border as u16,
        OptionValue::Float(border) => border as u16,
        _ => 5,
    };
    egui_overlay::start(HelloWorld {
        frame: 0,
        columns: 4,
        rows: 2,
        keeb: keeb,
        clicks: Vec::new(),
        target_client: current,
        monitor: monitor,
        border_width,
        margin,
        waybar_height,
    });
}
// const ICON_BYTES: &[u8] = include_bytes!("../../kitty_icon.png");
pub struct HelloWorld {
    pub frame: u64,
    pub columns: u16,
    pub rows: u16,
    pub keeb: Vec<Vec<String>>,
    pub clicks: Vec<(usize, usize)>,
    pub target_client: Client,
    pub monitor: Monitor,
    pub border_width: u16,
    pub margin: u16,
    pub waybar_height: u16,
}

fn calc_rowcol_bounds(clicks: &Vec<(usize, usize)>) -> (usize, usize, usize, usize) {
    if clicks.len() == 0 {
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

impl EguiOverlay for HelloWorld {
    fn gui_run(
        &mut self,
        egui_context: &egui::Context,
        _default_gfx_backend: &mut DefaultGfxBackend, //DefaultGfxBackend,
        glfw_backend: &mut egui_window_glfw_passthrough::GlfwBackend,
    ) {

        let evs: Vec<WindowEvent> = glfw_backend.frame_events.clone();
        if evs.len()>0 {
            println!("EVS: {:?}", &evs);
            for ev in evs{
                if let WindowEvent::Key(key, code, Action::Release, _) = ev{
                    println!("Matched key event with {:?}:{:?} as str ", key, key);
                    if let Some(name) = key.get_name(){
                        println!("That's a {}", name);
                        // OK, brute force which key it is.
                        for row in 0..self.rows as usize {
                            for col in 0..self.columns as usize {
                                if self.keeb[row][col].to_uppercase() == name.to_uppercase(){
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
            // let icon = image::load_from_memory(ICON_BYTES).unwrap().to_rgba8();
            {
                // if you enable `image` feature of glfw-passthrough crate, you can just use this
                // glfw_backend.window.set_icon(vec![icon]);

                // alternative api
                // useful when you don't want to enable image feature of glfw (maybe it pulls an older version of image crate leading to duplicate image crates in your dependency tree)
                /*
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
                */
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
                // ui.set_width(300.0);
                self.frame += 1;
                // ui.label(format!("current frame number: {}", self.frame));

                // sometimes, you want to see the borders to understand where the overlay is.
                let mut borders = glfw_backend.window.is_decorated();
                if ui.checkbox(&mut borders, "window borders").changed() {
                    glfw_backend.window.set_decorated(borders);
                }
                ui.horizontal(|ui| {
                    ui.label("Columns:");
                    ui.add(egui::Slider::new(&mut self.columns, 1..=9u16));
                    ui.label("Rows:");
                    ui.add(egui::Slider::new(&mut self.rows, 1..=3u16));
                });

                /*
                ui.label(format!(
                    "pixels_per_virtual_unit: {}",
                    glfw_backend.physical_pixels_per_virtual_unit
                ));
                ui.label(format!("window scale: {}", glfw_backend.scale));
                ui.label(format!("cursor pos x: {}", glfw_backend.cursor_pos[0]));
                ui.label(format!("cursor pos y: {}", glfw_backend.cursor_pos[1]));

                ui.label(format!(
                    "passthrough: {}",
                    glfw_backend.window.is_mouse_passthrough()
                ));
                */

                let (x0, y0, x1, y1) = calc_rowcol_bounds(&self.clicks);

                let button_width = (fbsize.0 as f32 - 32.) / self.columns as f32;
                let button_height = (fbsize.1 as f32 - 64.) / self.rows as f32;
                for row in 0..self.rows as usize {
                    ui.horizontal(|ui| {
                        for col in 0..self.columns as usize {
                            let mut button = egui::Button::new(self.keeb[row][col].clone())
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
                    // _default_gfx_backend.
                    let gaps_in = self.margin;
                    let gaps_out = self.margin;
                    let col_width = (self.monitor.width - 2 * gaps_out) / self.columns;
                    let row_height = (self.monitor.height
                        - (gaps_out * 2)
                        - self.waybar_height
                        - self.border_width)
                        / self.rows;
                    let new_width = (((x1 + 1) - x0) as u16 * col_width);
                    let new_height = ((y1 + 1) - y0) as u16 * row_height;
                    let left_ofs = self.monitor.x as u16 + gaps_out + (x0 as u16 * col_width);
                    let top_ofs = self.monitor.y as u16
                        + gaps_out
                        + self.waybar_height
                        + (y0 as u16 * row_height);
                    println!(
                        "MOVE TO X{},Y{} : W{},H{}",
                        left_ofs, top_ofs, new_width, new_height
                    );

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

                    glfw_backend.window.set_should_close(true)
                }
            });

        // here you decide if you want to be passthrough or not.
        if egui_context.wants_pointer_input() || egui_context.wants_keyboard_input() {
            // we need input, so we need the window to be NOT passthrough
            glfw_backend.set_passthrough(false);
        } else {
            // we don't care about input, so the window can be passthrough now
            glfw_backend.set_passthrough(true)
        }
        egui_context.request_repaint();
    }
}
