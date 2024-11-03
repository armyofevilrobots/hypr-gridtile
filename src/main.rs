use hyprland::{
    data::{Animations, Binds, Client, Clients, Monitor, Monitors, Workspace, Workspaces},
    dispatch::WindowIdentifier,
    shared::{HyprData, HyprDataActive, HyprDataActiveOptional},
};

use hyprland::dispatch;
use hyprland::dispatch::DispatchType::*;
use hyprland::dispatch::{
    Corner, Dispatch, DispatchType, FullscreenType, WorkspaceIdentifierWithSpecial,
};
use softbuffer::{Context, Surface};
use std::num::NonZeroU32;
use tiny_skia::{Color, FillRule, Paint, PathBuilder, Pixmap, Stroke, Transform};
use winit::event::{Event, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::raw_window_handle::DrmWindowHandle;
use winit::window::WindowBuilder;

fn main() -> hyprland::Result<()> {

    // Get the currently active client...
    let current = Client::get_active()?;
    let monitor = Monitor::get_active()?;
    let margin = 16;
    let waybar_height = 48;

    // Now we create a popup window that'll allow us to resize/position.
    let event_loop = EventLoop::new().unwrap();
    let window = WindowBuilder::new()
        .with_decorations(false)
        .with_transparent(true)
        // .with_fullscreen(Some(winit::window::Fullscreen::Borderless(None)))
        .build(&event_loop).unwrap();
    let context = Context::new(&window).unwrap();
    let mut surface = Surface::new(&context, &window).unwrap();
    let tmp_window = &window;
    let _ = event_loop
        .run(move |event, control_flow| {
            if let Event::WindowEvent { event, window_id } = event {
                match event {
                    WindowEvent::Focused(_) => {
                        if let Ok(Some(self_client)) = Client::get_active() {
                            // Everything I needed to do is now a winit window builder option.

                            dispatch!(
                                ToggleFloating,
                                Some(WindowIdentifier::Address(self_client.address.clone()))
                            )
                                .expect("Failed to float picker window.");
                            // dispatch!(ToggleFullscreen, FullscreenType::Maximize).expect("failed to fullscreen...");
                        }
                    },
                    WindowEvent::CloseRequested => {
                        control_flow.exit();
                    }
                    WindowEvent::RedrawRequested => {
                        let (width, height) = {
                            let size = tmp_window.inner_size();
                            (size.width as u32, size.height as u32)
                        };
                        surface
                            .resize(
                                NonZeroU32::new(width).unwrap(),
                                NonZeroU32::new(height).unwrap(),
                            )
                            .unwrap();

                        let mut pixmap = Pixmap::new(width, height).unwrap();
                        pixmap.fill(Color::from_rgba8(255,255,255,128));
                        let path = PathBuilder::from_circle(
                            (width / 2) as f32,
                            (height / 2) as f32,
                            (width.min(height) / 2) as f32,
                        )
                        .unwrap();
                        let mut paint = Paint::default();
                        paint.set_color_rgba8(0, 128, 128, 16);
                        // paint.set_color_rgba8(0, 0, 0 , 255);
                        pixmap.fill_path(
                            &path,
                            &paint,
                            FillRule::EvenOdd,
                            Transform::identity(),
                            None,
                        );
                        paint.set_color_rgba8(255, 0, 0, 16);
                        let mut stroke = Stroke::default();
                        stroke.width = 20.0;
                        pixmap.stroke_path(&path, &paint, &stroke, Transform::identity(), None);

                        let mut buffer = surface.buffer_mut().unwrap();
                        for index in 0..(width * height) as usize {
                            buffer[index] = pixmap.data()[index * 4 + 2] as u32
                                | (pixmap.data()[index * 4 + 1] as u32) << 8
                                | (pixmap.data()[index * 4] as u32) << 16;
                        }

                        buffer.present().unwrap();
                    }
                    _ => (),
                }
            }
        })
        .unwrap();

    // Force the current client to float.
    if let Some(target_client) = current{
        let window_id = WindowIdentifier::Address(target_client.address.clone());
        dispatch!(
            ToggleFloating,
            Some(window_id.clone())
        ).expect("This better be floating now eh?!");
        
        // Apply the position to the current client

        let new_width = monitor.width/2-(margin);
        let new_height = monitor.height-(margin*2)-waybar_height;
        let left_ofs = monitor.x as u16 + monitor.width/4+margin;
        let top_ofs = monitor.y as u16 + margin+waybar_height;
        println!("W/H/L/T: {}/{}/{}/{}", new_width, new_height, left_ofs, top_ofs);
        Dispatch::call(
            ResizeWindowPixel(dispatch::Position::Exact(new_width.try_into().unwrap(),
                                                        new_height.try_into().unwrap()),
                                         window_id.clone())
        ).expect("Should be resized naow.");
        Dispatch::call(
            MoveWindowPixel(dispatch::Position::Exact(left_ofs.try_into().unwrap(),
                                                        top_ofs.try_into().unwrap()
            ),
                                         window_id.clone())
        ).expect("Should be resized naow.");

    };




    Ok(())
}
