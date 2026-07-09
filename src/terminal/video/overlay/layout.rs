use crate::shared::{PORTRAIT_ASPECT, PORTRAIT_HEIGHT, PORTRAIT_WIDTH};
use crate::terminal::{
    layout::{CellRect, rect_to_pixels},
    metrics::{TerminalGrid, TerminalPixels, probe_terminal},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct OverlayLayout {
    pub(super) terminal_cols: u16,
    pub(super) terminal_rows: u16,
    pub(super) cell_rect: CellRect,
    pub(super) render_width: u32,
    pub(super) render_height: u32,
}

pub(super) fn compute_overlay_layout(
    terminal_cols: u16,
    terminal_rows: u16,
) -> Result<OverlayLayout, String> {
    let metrics = probe_terminal();
    let grid = metrics.grid.ok_or("terminal grid is unknown")?;
    let pixels = metrics.pixels.ok_or("terminal pixel size is unknown")?;
    let cell_rect = fit_portrait_rect(grid, pixels, terminal_cols, terminal_rows);
    let (_, _, pixel_width, pixel_height) = rect_to_pixels(grid, pixels, cell_rect);
    let (render_width, render_height) =
        cap_render_size(pixel_width, pixel_height, PORTRAIT_WIDTH, PORTRAIT_HEIGHT);
    Ok(OverlayLayout {
        terminal_cols,
        terminal_rows,
        cell_rect,
        render_width,
        render_height,
    })
}

pub(super) fn fit_portrait_rect(
    grid: TerminalGrid,
    pixels: TerminalPixels,
    terminal_cols: u16,
    terminal_rows: u16,
) -> CellRect {
    let cols = terminal_cols.min(grid.cols).max(1);
    let rows = terminal_rows.min(grid.rows).max(1);
    let cell_w = f64::from(pixels.width) / f64::from(grid.cols.max(1));
    let cell_h = f64::from(pixels.height) / f64::from(grid.rows.max(1));
    let available_w = f64::from(cols) * cell_w;
    let available_h = f64::from(rows) * cell_h;

    let (pixel_w, pixel_h) = if available_w / available_h.max(1.0) > PORTRAIT_ASPECT {
        (available_h * PORTRAIT_ASPECT, available_h)
    } else {
        (available_w, available_w / PORTRAIT_ASPECT)
    };
    let width = ((pixel_w / cell_w).round() as u16).clamp(1, cols);
    let height = ((pixel_h / cell_h).round() as u16).clamp(1, rows);

    CellRect {
        x: cols.saturating_sub(width) / 2,
        y: rows.saturating_sub(height) / 2,
        width,
        height,
    }
}

pub(super) fn cap_render_size(width: u32, height: u32, max_w: u32, max_h: u32) -> (u32, u32) {
    if width <= max_w && height <= max_h {
        return (width.max(1), height.max(1));
    }
    let scale = (max_w as f64 / width.max(1) as f64).min(max_h as f64 / height.max(1) as f64);
    (
        ((width as f64 * scale).round() as u32).max(1),
        ((height as f64 * scale).round() as u32).max(1),
    )
}
