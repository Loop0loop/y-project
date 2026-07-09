use crate::render::SceneKind;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Screen {
    Splash,
    Loading,
    Home,
    Training,
    CourtReplay,
    Dating,
    Result,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum TransitionPhase {
    FadeOut,
    FadeIn,
}

impl From<Screen> for SceneKind {
    fn from(screen: Screen) -> Self {
        match screen {
            Screen::Splash => SceneKind::Splash,
            Screen::Loading => SceneKind::Loading,
            Screen::Home => SceneKind::Home,
            Screen::Training => SceneKind::Training,
            Screen::CourtReplay => SceneKind::Court,
            Screen::Dating => SceneKind::Dating,
            Screen::Result => SceneKind::Result,
        }
    }
}
