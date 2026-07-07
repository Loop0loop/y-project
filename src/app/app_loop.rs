use std::{
    io::{self},
    time::{Duration, Instant},
};

use crossterm::{
    cursor::{Hide, Show},
    event::{self, Event},
    execute,
    terminal::{
        disable_raw_mode, enable_raw_mode, Clear, ClearType, EnterAlternateScreen,
        LeaveAlternateScreen,
    },
};

use super::{svg_presenter::SvgPresenter, text_view::draw_text, SpaApp};

const FRAME_TICK: Duration = Duration::from_millis(16);
const GAME_TICK: Duration = Duration::from_millis(50);

pub(crate) fn run_mvp_loop() -> Result<(), String> {
    run_terminal_loop(false)
}

pub(crate) fn run_mvp_svg_loop() -> Result<(), String> {
    run_terminal_loop(true)
}

fn run_terminal_loop(svg: bool) -> Result<(), String> {
    let mut stdout = io::stdout();
    enable_raw_mode().map_err(|error| error.to_string())?;
    if svg {
        execute!(stdout, EnterAlternateScreen, Clear(ClearType::All), Hide)
    } else {
        execute!(stdout, EnterAlternateScreen, Hide)
    }
    .map_err(|error| error.to_string())?;

    let mut guard = TerminalGuard;
    let result = if svg {
        run_svg_loop(&mut stdout)
    } else {
        run_text_loop(&mut stdout)
    };
    guard.restore(&mut stdout);
    result
}

fn run_text_loop(stdout: &mut io::Stdout) -> Result<(), String> {
    let mut app = SpaApp::new();
    let mut last_tick = Instant::now();
    loop {
        draw_text(stdout, &app)?;
        if input_requested_exit(&mut app)? {
            return Ok(());
        }
        tick_if_due(&mut app, &mut last_tick);
    }
}

fn run_svg_loop(stdout: &mut io::Stdout) -> Result<(), String> {
    let mut app = SpaApp::new();
    let mut presenter = SvgPresenter::new();
    let mut last_tick = Instant::now();
    loop {
        presenter.present(stdout, &app)?;
        if input_requested_exit(&mut app)? {
            presenter.delete(stdout)?;
            return Ok(());
        }
        tick_if_due(&mut app, &mut last_tick);
    }
}

fn input_requested_exit(app: &mut SpaApp) -> Result<bool, String> {
    if !event::poll(FRAME_TICK).map_err(|error| error.to_string())? {
        return Ok(false);
    }
    match event::read().map_err(|error| error.to_string())? {
        Event::Key(key) => Ok(app.on_key(key.code)),
        _ => Ok(false),
    }
}

fn tick_if_due(app: &mut SpaApp, last_tick: &mut Instant) {
    if last_tick.elapsed() >= GAME_TICK {
        app.tick();
        *last_tick = Instant::now();
    }
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
