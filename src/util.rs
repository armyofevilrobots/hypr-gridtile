use hyprland::data::Client;
use hyprland::dispatch::{FullscreenType, WindowIdentifier};
use hyprland::dispatch;
use hyprland::dispatch::Dispatch;
use hyprland::dispatch::{DispatchType, DispatchType::*};

pub(crate) fn calc_rowcol_bounds(clicks: &[(usize, usize)]) -> (usize, usize, usize, usize) {
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

pub(crate) fn init_logging() {
    use tracing_subscriber::{fmt, prelude::*, EnvFilter};
    // if RUST_LOG is not set, we will use the following filters
    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(
            EnvFilter::try_from_default_env().unwrap_or(EnvFilter::new("warn,wgpu=warn,naga=warn")),
        )
        .init();
}


pub(crate) fn force_fullscreen_window(target_client: &Client){
    if (target_client.fullscreen as u8) < 2 {
    dispatch!(ToggleFullscreen, FullscreenType::Real)
        .expect("This better be focused now eh?!");
    }
}


pub(crate) fn force_focus_window(target_client: &Client){
    let window_id: WindowIdentifier = WindowIdentifier::Address(target_client.address.clone());
    // if !target_client.focus_history_id==0 {
    dispatch!(FocusWindow, window_id.clone())
        .expect("This better be focused now eh?!");
    // }
}


pub(crate) fn force_float_window(target_client: &Client){
    let window_id: WindowIdentifier = WindowIdentifier::Address(target_client.address.clone());
    
    if !target_client.floating {
        dispatch!(ToggleFloating, Some(window_id.clone()))
            .expect("This better be floating now eh?!");
    }
}

pub(crate) fn move_and_resize_hypr_win(target_client: &Client, left_ofs: u16, top_ofs: u16, new_width: u16, new_height: u16) {
    let window_id: WindowIdentifier = WindowIdentifier::Address(target_client.address.clone());

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
        dispatch::Position::Exact(left_ofs.try_into().unwrap(), top_ofs.try_into().unwrap()),
        window_id.clone(),
    ))
    .expect("Should be resized naow.");
    Dispatch::call(MoveWindowPixel(
        dispatch::Position::Exact(left_ofs.try_into().unwrap(), top_ofs.try_into().unwrap()),
        window_id.clone(),
    ))
    .expect("Should be resized naow.");
}
