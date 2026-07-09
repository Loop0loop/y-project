use std::{io::Write, time::Duration};

use crate::{
    render::{RenderView, SceneKind},
    terminal::{
        kitty::{KittyImage, present_rgba},
        layout::{CellRect, rect_to_pixels},
        metrics::probe_terminal,
        session::{TerminalSession, wait_or_interrupt},
    },
};

use super::svg::{render_splash, render_view_rgba};

pub(crate) fn run_svg_demo() -> Result<(), String> {
    let metrics = probe_terminal();
    let grid = metrics.grid.ok_or("terminal grid is unknown")?;
    let pixels = metrics.pixels.ok_or("terminal pixel size is unknown")?;
    let cell_rect = CellRect {
        x: 2,
        y: 2,
        width: 40.min(grid.cols.saturating_sub(4)),
        height: 10.min(grid.rows.saturating_sub(4)),
    };
    let (_, _, pixel_width, pixel_height) = rect_to_pixels(grid, pixels, cell_rect);
    let rgba = render_view_rgba(pixel_width, pixel_height, &demo_view())?;
    let image = KittyImage {
        image_id: 424_243,
        placement_id: 8,
    };
    let mut terminal = TerminalSession::enter(false, false)?;
    terminal.register_image(image);
    present_rgba(
        terminal.stdout(),
        &rgba,
        pixel_width,
        pixel_height,
        cell_rect,
        &image,
        true,
    )?;
    write!(
        terminal.stdout(),
        "\x1b[{};{}Hsvg panel bbox={}x{}px cells={}x{} image_id={} placement_id={}   ",
        cell_rect.y + cell_rect.height + 2,
        cell_rect.x + 1,
        pixel_width,
        pixel_height,
        cell_rect.width,
        cell_rect.height,
        image.image_id,
        image.placement_id
    )
    .map_err(|error| error.to_string())?;
    terminal
        .stdout()
        .flush()
        .map_err(|error| error.to_string())?;
    wait_or_interrupt(Duration::from_secs(3))?;
    Ok(())
}

pub(crate) fn run_splash_demo() -> Result<(), String> {
    let metrics = probe_terminal();
    let grid = metrics.grid.ok_or("terminal grid is unknown")?;
    let pixels = metrics.pixels.ok_or("terminal pixel size is unknown")?;
    let cell_rect = CellRect {
        x: 0,
        y: 0,
        width: grid.cols,
        height: grid.rows,
    };
    let (_, _, pixel_width, pixel_height) = rect_to_pixels(grid, pixels, cell_rect);
    let rgba = render_splash(pixel_width, pixel_height, 68.0)?;
    let image = KittyImage {
        image_id: 424_244,
        placement_id: 9,
    };
    let mut terminal = TerminalSession::enter(false, false)?;
    terminal.register_image(image);
    present_rgba(
        terminal.stdout(),
        &rgba,
        pixel_width,
        pixel_height,
        cell_rect,
        &image,
        true,
    )?;
    terminal
        .stdout()
        .flush()
        .map_err(|error| error.to_string())?;
    wait_or_interrupt(Duration::from_secs(3))?;
    Ok(())
}

fn demo_view() -> RenderView {
    RenderView {
        scene: SceneKind::Training,
        phase_label: "TRAINING".to_string(),
        title: "TRAINING".to_string(),
        subtitle: "Logic Speed".to_string(),
        body: "SVG panel rendered from terminal pixel bbox".to_string(),
        side_title: "ACTIONS".to_string(),
        side_body: "Logic / Speech / Law / Nerve".to_string(),
        progress: 68.0,
        stats: crate::domain::AdvocateStats::default(),
        week: 1,
        focused_action: 0,
        ally_hp: 100,
        enemy_hp: 100,
        momentum: 0,
        ui_opacity: "1.00".to_string(),
        current_tab: 2,
    }
}
