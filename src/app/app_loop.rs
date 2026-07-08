use std::{
    io,
    time::{Duration, Instant},
};

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};

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
    event_requested_exit(app, event::read().map_err(|error| error.to_string())?)
}

fn event_requested_exit(app: &mut SpaApp, event: Event) -> Result<bool, String> {
    match event {
        Event::Key(key) => key_requested_exit(app, key),
        _ => Ok(false),
    }
}

fn key_requested_exit(app: &mut SpaApp, key: KeyEvent) -> Result<bool, String> {
    if key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL) {
        return Ok(true);
    }
    if key.code == KeyCode::Enter && key.modifiers != KeyModifiers::NONE {
        return Ok(false);
    }
    app.on_key(key.code)
}

fn tick_if_due(app: &mut SpaApp, last_tick: &mut Instant) {
    if last_tick.elapsed() >= GAME_TICK {
        app.tick();
        *last_tick = Instant::now();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::phase::GamePhase;

    fn app_state(app: &SpaApp) -> (Screen, GamePhase, String, u32, u32, usize, usize) {
        (
            app.screen(),
            app.phase(),
            app.input().to_string(),
            app.splash_progress(),
            app.loading_progress(),
            app.focused_action(),
            app.shown_court_logs(),
        )
    }

    #[test]
    fn resize_event_does_not_mutate_app_lifecycle() {
        let mut app = SpaApp::new_with_screen(Screen::Training).unwrap();
        let before = app_state(&app);

        assert!(!event_requested_exit(&mut app, Event::Resize(120, 40)).unwrap());
        assert_eq!(app_state(&app), before);
    }

    #[test]
    fn modified_enter_is_not_app_start() {
        let mut app = SpaApp::new_with_screen(Screen::Splash).unwrap();

        assert!(
            !key_requested_exit(
                &mut app,
                KeyEvent::new(KeyCode::Enter, KeyModifiers::CONTROL)
            )
            .unwrap()
        );
        assert!(matches!(app.screen(), Screen::Splash));
        assert_eq!(app.splash_progress(), 0);
    }

    #[test]
    fn ctrl_c_exits_from_raw_mode_loop() {
        let mut app = SpaApp::new_with_screen(Screen::Splash).unwrap();

        assert!(
            key_requested_exit(
                &mut app,
                KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL)
            )
            .unwrap()
        );
    }
}
