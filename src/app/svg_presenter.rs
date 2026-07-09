use std::{
    io::{self, Write},
    time::{Duration, Instant},
};

use crate::{
    render::{RenderView, SceneKind, render_view_rgba},
    shared::PORTRAIT_ASPECT,
    terminal::{
        kitty::{KittyImage, present_rgba},
        layout::{CellRect, rect_to_pixels},
        metrics::probe_terminal,
    },
};

use super::{Screen, SpaApp};

const METRICS_REFRESH: Duration = Duration::from_millis(250);
const ANIMATED_RENDER_STEP: Duration = Duration::from_millis(16);

#[derive(Debug, Clone, PartialEq)]
struct RenderKey {
    view: RenderView,
    pixel_width: u32,
    pixel_height: u32,
    cell_rect: CellRect,
}

pub(super) struct SvgPresenter {
    image: KittyImage,
    last_key: Option<RenderKey>,
    last_frame: Option<TerminalFrame>,
    last_metrics_check: Option<Instant>,
    last_render_at: Option<Instant>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct TerminalFrame {
    panel: CellRect,
    render_width: u32,
    render_height: u32,
}

impl SvgPresenter {
    pub(super) fn new() -> Self {
        Self {
            image: KittyImage {
                image_id: 525_200,
                placement_id: 22,
            },
            last_key: None,
            last_frame: None,
            last_metrics_check: None,
            last_render_at: None,
        }
    }

    pub(super) fn image(&self) -> KittyImage {
        self.image
    }

    pub(super) fn invalidate_frame(&mut self) {
        self.last_frame = None;
        self.last_metrics_check = None;
    }

    pub(super) fn present(&mut self, stdout: &mut io::Stdout, app: &SpaApp) -> Result<(), String> {
        let frame = self.frame(app.screen())?;
        let view = app.view_model();
        let key = RenderKey {
            view,
            pixel_width: frame.render_width,
            pixel_height: frame.render_height,
            cell_rect: frame.panel,
        };

        if self.should_render(&key) {
            let rgba = render_view_rgba(frame.render_width, frame.render_height, &key.view)?;
            present_rgba(
                stdout,
                &rgba,
                frame.render_width,
                frame.render_height,
                frame.panel,
                &self.image,
                true,
            )?;
            self.last_key = Some(key);
            self.last_render_at = Some(Instant::now());
        }

        draw_overlay(stdout, app, frame.panel)
    }

    fn frame(&mut self, screen: Screen) -> Result<TerminalFrame, String> {
        let now = Instant::now();
        if self
            .last_metrics_check
            .is_some_and(|last| now.duration_since(last) < METRICS_REFRESH)
            && let Some(frame) = self.last_frame
        {
            return Ok(frame);
        }

        let metrics = probe_terminal();
        let grid = metrics.grid.ok_or("terminal grid is unknown")?;
        let pixels = metrics.pixels.ok_or("terminal pixel size is unknown")?;
        let panel = panel_rect(screen, grid, pixels);
        let (_, _, pixel_width, pixel_height) = rect_to_pixels(grid, pixels, panel);
        let frame = TerminalFrame {
            panel,
            render_width: pixel_width.max(1),
            render_height: pixel_height.max(1),
        };
        self.last_frame = Some(frame);
        self.last_metrics_check = Some(now);
        Ok(frame)
    }

    fn should_render(&self, key: &RenderKey) -> bool {
        let Some(last_key) = self.last_key.as_ref() else {
            return true;
        };
        if last_key == key {
            return false;
        }
        if last_key.cell_rect != key.cell_rect
            || last_key.pixel_width != key.pixel_width
            || last_key.pixel_height != key.pixel_height
            || last_key.view.scene != key.view.scene
        {
            return true;
        }
        if matches!(key.view.scene, SceneKind::Training | SceneKind::Result) {
            return true;
        }

        self.last_render_at
            .is_none_or(|last| last.elapsed() >= ANIMATED_RENDER_STEP)
    }
}

fn panel_rect(
    screen: Screen,
    grid: crate::terminal::metrics::TerminalGrid,
    pixels: crate::terminal::metrics::TerminalPixels,
) -> CellRect {
    let cols = grid.cols;
    let rows = grid.rows;
    if screen == Screen::Home {
        return fit_portrait_panel(grid, pixels, cols, rows.saturating_sub(2).max(1));
    }
    if portrait(screen) {
        return fit_portrait_panel(grid, pixels, cols.min(75), rows.saturating_sub(2).min(45));
    }
    CellRect {
        x: 0,
        y: 0,
        width: cols,
        height: rows.saturating_sub(2).max(1),
    }
}

fn fit_portrait_panel(
    grid: crate::terminal::metrics::TerminalGrid,
    pixels: crate::terminal::metrics::TerminalPixels,
    cols: u16,
    rows: u16,
) -> CellRect {
    let cols = cols.max(1);
    let rows = rows.max(1);
    let cell_w = f64::from(pixels.width) / f64::from(grid.cols.max(1));
    let cell_h = f64::from(pixels.height) / f64::from(grid.rows.max(1));
    let available_w = f64::from(cols) * cell_w;
    let available_h = f64::from(rows) * cell_h;
    let (pixel_w, pixel_h) = if available_w / available_h > PORTRAIT_ASPECT {
        (available_h * PORTRAIT_ASPECT, available_h)
    } else {
        (available_w, available_w / PORTRAIT_ASPECT)
    };
    let width = ((pixel_w / cell_w).round() as u16).clamp(1, cols);
    let height = ((pixel_h / cell_h).round() as u16).clamp(1, rows);

    CellRect {
        x: grid.cols.saturating_sub(width) / 2,
        y: grid.rows.saturating_sub(height + 2) / 2,
        width,
        height,
    }
}

fn portrait(screen: Screen) -> bool {
    matches!(
        screen,
        Screen::Splash | Screen::Loading | Screen::Home | Screen::Training
    )
}

fn draw_overlay(stdout: &mut io::Stdout, app: &SpaApp, panel: CellRect) -> Result<(), String> {
    write!(
        stdout,
        "\x1b[{};{}HProject-Y SPA SVG Loop | Up/Down Enter | q/Esc quit",
        panel.y + panel.height + 1,
        panel.x + 2
    )
    .map_err(|error| error.to_string())?;
    if matches!(app.screen(), Screen::Dating) {
        write!(
            stdout,
            "\x1b[{};{}H> {}",
            panel.y + panel.height + 2,
            panel.x + 2,
            app.input()
        )
        .map_err(|error| error.to_string())?;
    }
    stdout.flush().map_err(|error| error.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn presenter_key_changes_when_view_changes() {
        let mut app = SpaApp::new_with_screen(Screen::Training).unwrap();
        let panel = CellRect {
            x: 0,
            y: 0,
            width: 80,
            height: 24,
        };
        let first = RenderKey {
            view: app.view_model(),
            pixel_width: 800,
            pixel_height: 480,
            cell_rect: panel,
        };
        app.on_key(crossterm::event::KeyCode::Down).unwrap();
        let second = RenderKey {
            view: app.view_model(),
            pixel_width: 800,
            pixel_height: 480,
            cell_rect: panel,
        };
        assert_ne!(first, second);
    }

    #[test]
    fn portrait_panel_keeps_home_aspect() {
        let rect = fit_portrait_panel(
            crate::terminal::metrics::TerminalGrid {
                cols: 160,
                rows: 60,
            },
            crate::terminal::metrics::TerminalPixels {
                width: 1600,
                height: 1200,
            },
            75,
            45,
        );
        let aspect = f64::from(rect.width) * 10.0 / (f64::from(rect.height) * 20.0);
        assert!((aspect - PORTRAIT_ASPECT).abs() < 0.03);
    }

    #[test]
    fn home_panel_uses_full_terminal_height() {
        let rect = panel_rect(
            Screen::Home,
            crate::terminal::metrics::TerminalGrid {
                cols: 160,
                rows: 60,
            },
            crate::terminal::metrics::TerminalPixels {
                width: 1600,
                height: 1200,
            },
        );
        assert_eq!(rect.height, 58);
    }
}
