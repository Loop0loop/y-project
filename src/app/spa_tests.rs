use super::*;
use crate::domain::phase::GamePhase;
use crossterm::event::KeyCode;

fn lifecycle_is_valid(app: &SpaApp) -> bool {
    matches!(
        (app.screen(), app.phase()),
        (
            Screen::Splash | Screen::Loading | Screen::Training,
            GamePhase::Training
        ) | (Screen::Home, GamePhase::Training)
            | (Screen::CourtReplay | Screen::Dating, GamePhase::Dating)
            | (Screen::Result, GamePhase::Result)
    )
}

#[test]
fn enter_training_starts_court_replay() {
    let mut app = SpaApp::new_with_screen(Screen::Training).unwrap();
    assert!(!app.on_key(KeyCode::Enter).unwrap());
    assert!(matches!(app.screen(), Screen::CourtReplay));
    assert_eq!(app.phase(), GamePhase::Dating);
    assert_eq!(app.court_log_len(), 3);
    assert!(lifecycle_is_valid(&app));
}

#[test]
fn builds_view_model_from_training_focus() {
    let mut app = SpaApp::new_with_screen(Screen::Training).unwrap();
    app.on_key(KeyCode::Down).unwrap();
    app.on_key(KeyCode::Down).unwrap();
    let view = app.view_model();
    assert_eq!(view.title, "TRAINING");
    assert!(view.subtitle.contains("Law Study"));
    assert!(view.body.contains("LOG"));
}

#[test]
fn enter_splash_opens_home() {
    let mut app = SpaApp::new_with_screen(Screen::Splash).unwrap();
    assert!(!app.on_key(KeyCode::Enter).unwrap());
    assert!(matches!(app.screen(), Screen::Home));
    assert!(lifecycle_is_valid(&app));
}

#[test]
fn screen_and_domain_phase_move_together() {
    let mut app = SpaApp::new_with_screen(Screen::Training).unwrap();
    assert!(lifecycle_is_valid(&app));

    app.on_key(KeyCode::Enter).unwrap();
    assert!(matches!(app.screen(), Screen::CourtReplay));
    assert!(lifecycle_is_valid(&app));

    app.on_key(KeyCode::Enter).unwrap();
    assert!(matches!(app.screen(), Screen::Dating));
    assert!(lifecycle_is_valid(&app));

    app.on_key(KeyCode::Enter).unwrap();
    assert!(matches!(app.screen(), Screen::Result));
    assert!(lifecycle_is_valid(&app));
}

#[test]
fn rejects_non_initial_start_screens() {
    for screen in [Screen::CourtReplay, Screen::Dating, Screen::Result] {
        assert!(SpaApp::new_with_screen(screen).is_err());
    }
}
