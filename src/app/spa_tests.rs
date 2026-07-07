use super::*;
use crate::domain::phase::GamePhase;
use crossterm::event::KeyCode;

#[test]
fn enter_training_starts_court_replay() {
    let mut app = SpaApp::new();
    app.screen = Screen::Training;
    assert!(!app.on_key(KeyCode::Enter));
    assert!(matches!(app.screen, Screen::CourtReplay));
    assert_eq!(app.session.phase, GamePhase::Dating);
    assert_eq!(app.session.court_log().len(), 3);
}

#[test]
fn builds_view_model_from_training_focus() {
    let mut app = SpaApp::new();
    app.screen = Screen::Training;
    app.focused_action = 2;
    let view = app.view_model();
    assert_eq!(view.title, "TRAINING");
    assert!(view.subtitle.contains("Law Study"));
    assert!(view.body.contains("LOG"));
}
