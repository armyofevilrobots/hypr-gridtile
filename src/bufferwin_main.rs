use egui_overlay::egui_render_three_d::ThreeDBackend as DefaultGfxBackend;
use egui_overlay::EguiOverlay;
use egui_window_glfw_passthrough::glfw::{Action, Key, WindowEvent};
use hyprland::dispatch;
use hyprland::dispatch::Dispatch;
use hyprland::dispatch::DispatchType;
use hyprland::{
    data::Client,
    dispatch::WindowIdentifier,
    shared::HyprDataActiveOptional,
};


fn main() {
    use tracing_subscriber::{fmt, prelude::*, EnvFilter};
    // if RUST_LOG is not set, we will use the following filters
    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(
            EnvFilter::try_from_default_env()
                .unwrap_or(EnvFilter::new("debug,glium=debug,wgpu=warn,naga=warn")),
        )
        .init();
    egui_overlay::start(BufferWinState {
        frame: 0,
    });
}
const ICON_BYTES: &[u8] = include_bytes!("../resources/hypr-gridtile.png");

pub struct BufferWinState {
    pub frame: u64,
}


impl EguiOverlay for BufferWinState {
    fn gui_run(
        &mut self,
        egui_context: &egui::Context,
        _default_gfx_backend: &mut DefaultGfxBackend, //DefaultGfxBackend,
        glfw_backend: &mut egui_window_glfw_passthrough::GlfwBackend,
    ) {

        // first frame logic
        if self.frame == 0 {

            if let Ok(Some(self_client)) = Client::get_active() {
                if self_client.floating{
                    dispatch!(
                        ToggleFloating,
                        Some(WindowIdentifier::Address(self_client.address.clone()))
                    )
                        .expect("Failed to float picker window.");
                }
            }
            glfw_backend.set_title("Hypr-BufferWin".to_string());
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


        let evs: Vec<WindowEvent> = glfw_backend.frame_events.clone();
        if !evs.is_empty() {
            for ev in evs {
                if let WindowEvent::Key(key, _code, Action::Press, _) = ev {
                    if key == Key::Escape || key == Key::Q || key == Key::X {
                        glfw_backend.window.set_should_close(true);
                    }
                }
            }
        }

        egui_context.request_repaint();
    }
}
