use std::{
    io::{self, Write},
    time::{Duration, Instant},
};

use crossterm::{
    cursor::{Hide, MoveTo, Show},
    event::{self, Event},
    execute,
    terminal::{
        Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode,
        enable_raw_mode,
    },
};

use crate::{
    domain::{AdvocateStats, TRAINING_ACTIONS},
    render::{PanelSpec, render_svg_panel},
    terminal::{
        kitty::{KittyImage, delete_image, present_rgba},
        layout::{CellRect, rect_to_pixels},
        metrics::probe_terminal,
    },
};

use super::{AppViewModel, FAKE_RESPONSE, Screen, SpaApp};

const FRAME_TICK: Duration = Duration::from_millis(16);
const GAME_TICK: Duration = Duration::from_millis(50);
const METRICS_REFRESH: Duration = Duration::from_millis(250);
const ANIMATED_RENDER_STEP: Duration = Duration::from_millis(33);
const MAX_RENDER_WIDTH: u32 = 960;
const MAX_RENDER_HEIGHT: u32 = 540;

#[derive(Debug, Clone, PartialEq, Eq)]
struct RenderKey {
    view: AppViewModel,
    pixel_width: u32,
    pixel_height: u32,
    cell_rect: CellRect,
}

struct SvgPresenter {
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
    fn new() -> Self {
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

    fn present(&mut self, stdout: &mut io::Stdout, app: &SpaApp) -> Result<(), String> {
        let frame = self.frame()?;
        let view = app.view_model();
        let key = RenderKey {
            view,
            pixel_width: frame.render_width,
            pixel_height: frame.render_height,
            cell_rect: frame.panel,
        };

        if self.should_render(&key) {
            let rgba = render_svg_panel(
                frame.render_width,
                frame.render_height,
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
                },
            )?;
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

    fn delete(&self, stdout: &mut io::Stdout) -> Result<(), String> {
        delete_image(stdout, &self.image)
    }

    fn frame(&mut self) -> Result<TerminalFrame, String> {
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
        let panel = CellRect {
            x: 0,
            y: 0,
            width: grid.cols,
            height: grid.rows.saturating_sub(2).max(1),
        };
        let (_, _, pixel_width, pixel_height) = rect_to_pixels(grid, pixels, panel);
        let (render_width, render_height) = capped_render_size(pixel_width, pixel_height);
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

pub(crate) fn run_mvp_loop() -> Result<(), String> {
    let mut stdout = io::stdout();
    enable_raw_mode().map_err(|error| error.to_string())?;
    execute!(stdout, EnterAlternateScreen, Hide).map_err(|error| error.to_string())?;

    let mut guard = TerminalGuard;
    let result = run_text_loop(&mut stdout);
    guard.restore(&mut stdout);
    result
}

pub(crate) fn run_mvp_svg_loop() -> Result<(), String> {
    let mut stdout = io::stdout();
    enable_raw_mode().map_err(|error| error.to_string())?;
    execute!(stdout, EnterAlternateScreen, Clear(ClearType::All), Hide)
        .map_err(|error| error.to_string())?;

    let mut guard = TerminalGuard;
    let result = run_svg_loop(&mut stdout);
    guard.restore(&mut stdout);
    result
}

fn run_text_loop(stdout: &mut io::Stdout) -> Result<(), String> {
    let mut app = SpaApp::new();
    let mut last_tick = Instant::now();
    loop {
        draw_text(stdout, &app)?;
        if event::poll(FRAME_TICK).map_err(|error| error.to_string())? {
            if let Event::Key(key) = event::read().map_err(|error| error.to_string())? {
                if app.on_key(key.code) {
                    return Ok(());
                }
            }
        }
        if last_tick.elapsed() >= GAME_TICK {
            app.tick();
            last_tick = Instant::now();
        }
    }
}

fn run_svg_loop(stdout: &mut io::Stdout) -> Result<(), String> {
    let mut app = SpaApp::new();
    let mut presenter = SvgPresenter::new();
    let mut last_tick = Instant::now();
    loop {
        presenter.present(stdout, &app)?;
        if event::poll(FRAME_TICK).map_err(|error| error.to_string())? {
            if let Event::Key(key) = event::read().map_err(|error| error.to_string())? {
                if app.on_key(key.code) {
                    presenter.delete(stdout)?;
                    return Ok(());
                }
            }
        }
        if last_tick.elapsed() >= GAME_TICK {
            app.tick();
            last_tick = Instant::now();
        }
    }
}

fn draw_text(stdout: &mut io::Stdout, app: &SpaApp) -> Result<(), String> {
    execute!(stdout, MoveTo(0, 0), Clear(ClearType::All)).map_err(|error| error.to_string())?;
    writeln!(stdout, "Project-Y MVP Loop  |  q/Esc quit").map_err(|error| error.to_string())?;
    writeln!(stdout, "phase={:?}", app.session.phase).map_err(|error| error.to_string())?;
    writeln!(stdout).map_err(|error| error.to_string())?;

    match app.screen {
        Screen::Splash => draw_splash(stdout, app),
        Screen::Training => draw_training(stdout, app),
        Screen::CourtReplay => draw_court(stdout, app),
        Screen::Dating => draw_dating(stdout, app),
        Screen::Result => draw_result(stdout, app),
    }?;

    stdout.flush().map_err(|error| error.to_string())
}

fn draw_splash(stdout: &mut io::Stdout, app: &SpaApp) -> Result<(), String> {
    writeln!(
        stdout,
        "[Splash Screen] Loading... {}%",
        app.splash_progress
    )
    .map_err(|error| error.to_string())?;
    writeln!(stdout, "Press Enter or Space or Down arrow to skip")
        .map_err(|error| error.to_string())
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

fn draw_training(stdout: &mut io::Stdout, app: &SpaApp) -> Result<(), String> {
    writeln!(stdout, "[Training] Up/Down select, Enter confirm")
        .map_err(|error| error.to_string())?;
    for (index, action) in TRAINING_ACTIONS.iter().enumerate() {
        let marker = if index == app.focused_action {
            ">"
        } else {
            " "
        };
        writeln!(stdout, "{marker} {}", action.label).map_err(|error| error.to_string())?;
    }
    writeln!(stdout).map_err(|error| error.to_string())?;
    write_stats(stdout, app.session.stats())
}

fn draw_court(stdout: &mut io::Stdout, app: &SpaApp) -> Result<(), String> {
    writeln!(stdout, "[Court Replay] Enter skip").map_err(|error| error.to_string())?;
    for line in app.session.court_log().iter().take(app.shown_court_logs) {
        writeln!(stdout, "- {line}").map_err(|error| error.to_string())?;
    }
    if app.shown_court_logs == app.session.court_log().len() {
        writeln!(stdout, "result={:?}", app.session.court_result())
            .map_err(|error| error.to_string())?;
    }
    Ok(())
}

fn draw_dating(stdout: &mut io::Stdout, app: &SpaApp) -> Result<(), String> {
    writeln!(stdout, "[Dating] type message, Enter finish").map_err(|error| error.to_string())?;
    let response: String = FAKE_RESPONSE
        .chars()
        .take(app.visible_response_chars())
        .collect();
    for line in wrap_text(&response, 54) {
        writeln!(stdout, "Furina: {line}").map_err(|error| error.to_string())?;
    }
    writeln!(stdout).map_err(|error| error.to_string())?;
    writeln!(stdout, "> {}", app.input).map_err(|error| error.to_string())
}

fn draw_result(stdout: &mut io::Stdout, app: &SpaApp) -> Result<(), String> {
    writeln!(stdout, "[Result] Enter exit").map_err(|error| error.to_string())?;
    writeln!(stdout, "court={:?}", app.session.court_result())
        .map_err(|error| error.to_string())?;
    writeln!(stdout, "transcript_len={}", app.session.transcript_len())
        .map_err(|error| error.to_string())
}

fn write_stats(stdout: &mut io::Stdout, stats: AdvocateStats) -> Result<(), String> {
    for (label, value) in [
        ("Logic Speed", stats.logic_speed),
        ("Mental Stamina", stats.mental_stamina),
        ("Speech Power", stats.speech_power),
        ("Guts", stats.guts),
        ("Intellect", stats.intellect),
    ] {
        writeln!(stdout, "{label:15} {}", bar(value)).map_err(|error| error.to_string())?;
    }
    Ok(())
}

fn bar(value: u16) -> String {
    let filled = (usize::from(value.min(100)) * 20) / 100;
    format!(
        "[{}{}] {value:03}",
        "#".repeat(filled),
        ".".repeat(20 - filled)
    )
}

fn capped_render_size(width: u32, height: u32) -> (u32, u32) {
    if width <= MAX_RENDER_WIDTH && height <= MAX_RENDER_HEIGHT {
        return (width.max(1), height.max(1));
    }

    let width_scale = MAX_RENDER_WIDTH as f64 / width.max(1) as f64;
    let height_scale = MAX_RENDER_HEIGHT as f64 / height.max(1) as f64;
    let scale = width_scale.min(height_scale);
    (
        ((width as f64 * scale).round() as u32).max(1),
        ((height as f64 * scale).round() as u32).max(1),
    )
}

fn wrap_text(text: &str, width: usize) -> Vec<String> {
    let mut lines = Vec::new();
    let mut current = String::new();
    for ch in text.chars() {
        if current.chars().count() >= width {
            lines.push(std::mem::take(&mut current));
        }
        current.push(ch);
    }
    if !current.is_empty() {
        lines.push(current);
    }
    lines
}

struct TerminalGuard;

impl TerminalGuard {
    fn restore(&mut self, stdout: &mut io::Stdout) {
        let _ = execute!(stdout, Show, LeaveAlternateScreen);
        let _ = disable_raw_mode();
    }
}

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        let _ = disable_raw_mode();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wraps_korean_and_english_without_losing_chars() {
        let text = "abc가나다def";
        let lines = wrap_text(text, 4);
        assert_eq!(lines.concat(), text);
        assert!(lines.iter().all(|line| line.chars().count() <= 4));
    }

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
        assert_eq!(capped_render_size(800, 450), (800, 450));
        assert_eq!(capped_render_size(3024, 1964), (831, 540));
    }
}
