const ICON_BYTES: &[u8] = include_bytes!("../resources/hypr-gridtile.png");

pub(crate) fn glfw_set_icon(glfw_backend: &mut egui_window_glfw_passthrough::GlfwBackend){
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
