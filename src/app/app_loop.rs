use std::{
    io,
    time::{Duration, Instant},
};

use crossterm::event::{self, Event, KeyCode, KeyModifiers};

use crate::terminal::session::TerminalSession;

use super::{Screen, SpaApp, svg_presenter::SvgPresenter, text_view::draw_text};

const FRAME_TICK: Duration = Duration::from_millis(16);
const GAME_TICK: Duration = Duration::from_millis(50);

pub(crate) fn run_mvp_loop() -> Result<(), String> {
    run_terminal_loop(false, Screen::Splash)
}

pub(crate) fn run_mvp_svg_loop(start_screen: Screen) -> Result<(), String> {
    run_terminal_loop(true, start_screen)
}

fn run_terminal_loop(svg: bool, start_screen: Screen) -> Result<(), String> {
    let mut terminal = TerminalSession::enter(svg, false)?;
    if svg {
        run_svg_loop(&mut terminal, start_screen)
    } else {
        run_text_loop(terminal.stdout(), start_screen)
    }
}

fn run_text_loop(stdout: &mut io::Stdout, start_screen: Screen) -> Result<(), String> {
    let mut app = SpaApp::new_with_screen(start_screen)?;
    let mut last_tick = Instant::now();
    loop {
        draw_text(stdout, &app)?;
        if input_requested_exit(&mut app)? {
            return Ok(());
        }
        tick_if_due(&mut app, &mut last_tick);
    }
}

fn run_svg_loop(terminal: &mut TerminalSession, start_screen: Screen) -> Result<(), String> {
    let mut app = SpaApp::new_with_screen(start_screen)?;
    let mut presenter = SvgPresenter::new();
    terminal.register_image(presenter.image());
    let mut last_tick = Instant::now();
    loop {
        presenter.present(terminal.stdout(), &app)?;
        if input_requested_exit(&mut app)? {
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
        Event::Key(key)
            if key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL) =>
        {
            Ok(true)
        }
        Event::Key(key) => app.on_key(key.code),
        _ => Ok(false),
    }
}

fn tick_if_due(app: &mut SpaApp, last_tick: &mut Instant) {
    if last_tick.elapsed() >= GAME_TICK {
        app.tick();
        *last_tick = Instant::now();
    }
}
