use std::{
    io::{self, Write},
    time::{Duration, Instant},
};

use crate::{
    render::{render_svg_panel, PanelSpec},
    terminal::{
        kitty::{delete_image, present_rgba, KittyImage},
        layout::{rect_to_pixels, CellRect},
        metrics::probe_terminal,
    },
};

use super::{AppViewModel, Screen, SpaApp};

const METRICS_REFRESH: Duration = Duration::from_millis(250);
const ANIMATED_RENDER_STEP: Duration = Duration::from_millis(33);

#[derive(Debug, Clone, PartialEq, Eq)]
struct RenderKey {
    view: AppViewModel,
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

    pub(super) fn present(&mut self, stdout: &mut io::Stdout, app: &SpaApp) -> Result<(), String> {
        let frame = self.frame(app.screen)?;
        let view = app.view_model();
        let key = RenderKey {
            view,
            pixel_width: frame.render_width,
            pixel_height: frame.render_height,
            cell_rect: frame.panel,
        };

        if self.should_render(&key) {
            let rgba = render_svg_panel(frame.render_width, frame.render_height, panel_spec(&key))?;
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

    pub(super) fn delete(&self, stdout: &mut io::Stdout) -> Result<(), String> {
        delete_image(stdout, &self.image)
    }

    fn frame(&mut self, screen: Screen) -> Result<TerminalFrame, String> {
        let now = Instant::now();
        if self
            .last_metrics_check
            .is_some_and(|last| now.duration_since(last) < METRICS_REFRESH)
        {
            if let Some(frame) = self.last_frame {
                return Ok(frame);
            }
        }

        let metrics = probe_terminal();
        let grid = metrics.grid.ok_or("terminal grid is unknown")?;
        let pixels = metrics.pixels.ok_or("terminal pixel size is unknown")?;
        let panel = panel_rect(screen, grid.cols, grid.rows);
        let (_, _, pixel_width, pixel_height) = rect_to_pixels(grid, pixels, panel);
        let (max_w, max_h) = if portrait(screen) {
            (600, 900)
        } else {
            (960, 540)
        };
        let (render_width, render_height) =
            capped_render_size(pixel_width, pixel_height, max_w, max_h);
        let frame = TerminalFrame {
            panel,
            render_width,
            render_height,
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
            || last_key.view.screen != key.view.screen
        {
            return true;
        }
        if matches!(key.view.screen, Screen::Training | Screen::Result) {
            return true;
        }

        self.last_render_at
            .is_none_or(|last| last.elapsed() >= ANIMATED_RENDER_STEP)
    }
}

fn panel_spec<'a>(key: &'a RenderKey) -> PanelSpec<'a> {
    PanelSpec {
        phase_label: &key.view.phase_label,
        title: &key.view.title,
        subtitle: &key.view.subtitle,
        body: &key.view.body,
        side_title: &key.view.side_title,
        side_body: &key.view.side_body,
        bar_percent: key.view.progress,
        screen: key.view.screen,
        stats: key.view.stats,
        week: key.view.week,
        focused_action: key.view.focused_action,
        ally_hp: key.view.ally_hp,
        enemy_hp: key.view.enemy_hp,
        momentum: key.view.momentum,
    }
}

fn panel_rect(screen: Screen, cols: u16, rows: u16) -> CellRect {
    if portrait(screen) {
        let width = 75.min(cols);
        let height = 45.min(rows.saturating_sub(2).max(1));
        return CellRect {
            x: cols.saturating_sub(width) / 2,
            y: rows.saturating_sub(height + 2) / 2,
            width,
            height,
        };
    }
    CellRect {
        x: 0,
        y: 0,
        width: cols,
        height: rows.saturating_sub(2).max(1),
    }
}

fn portrait(screen: Screen) -> bool {
    matches!(screen, Screen::Splash | Screen::Training)
}

fn draw_overlay(stdout: &mut io::Stdout, app: &SpaApp, panel: CellRect) -> Result<(), String> {
    write!(
        stdout,
        "\x1b[{};{}HProject-Y SPA SVG Loop | Up/Down Enter | q/Esc quit",
        panel.y + panel.height + 1,
        panel.x + 2
    )
    .map_err(|error| error.to_string())?;
    if matches!(app.screen, Screen::Dating) {
        write!(
            stdout,
            "\x1b[{};{}H> {}",
            panel.y + panel.height + 2,
            panel.x + 2,
            app.input
        )
        .map_err(|error| error.to_string())?;
    }
    stdout.flush().map_err(|error| error.to_string())
}

fn capped_render_size(width: u32, height: u32, max_w: u32, max_h: u32) -> (u32, u32) {
    if width <= max_w && height <= max_h {
        return (width.max(1), height.max(1));
    }

    let width_scale = max_w as f64 / width.max(1) as f64;
    let height_scale = max_h as f64 / height.max(1) as f64;
    let scale = width_scale.min(height_scale);
    (
        ((width as f64 * scale).round() as u32).max(1),
        ((height as f64 * scale).round() as u32).max(1),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn presenter_key_changes_when_view_changes() {
        let mut app = SpaApp::new();
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
        app.focused_action = 1;
        let second = RenderKey {
            view: app.view_model(),
            pixel_width: 800,
            pixel_height: 480,
            cell_rect: panel,
        };
        assert_ne!(first, second);
    }

    #[test]
    fn caps_render_size_without_changing_aspect_too_much() {
        assert_eq!(capped_render_size(800, 450, 960, 540), (800, 450));
        assert_eq!(capped_render_size(3024, 1964, 960, 540), (831, 540));
    }
}
