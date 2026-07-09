use std::time::{Duration, Instant};

use crossterm::event::KeyCode;

use super::screen::{Screen, TransitionPhase};
#[cfg(test)]
use crate::domain::phase::GamePhase;
use crate::domain::{DatingEndReason, GameSession, TRAINING_ACTIONS};
use crate::shared::{ease_out, loading_tip};

const COURT_LOG_STEP: Duration = Duration::from_millis(520);
const SPLASH_PROGRESS_DURATION: Duration = Duration::from_millis(1600);
const LOADING_PROGRESS_DURATION: Duration = Duration::from_secs(4);
pub(crate) const FAKE_RESPONSE: &str =
    "흠, 승부가 완벽했다고는 못 하겠지만... 네 논리에는 확실히 반짝이는 부분이 있었어.";

pub(crate) struct SpaApp {
    pub(crate) session: GameSession,
    pub(crate) screen: Screen,
    pub(crate) focused_action: usize,
    pub(crate) shown_court_logs: usize,
    pub(crate) last_court_log: Instant,
    pub(crate) input: String,
    pub(crate) visible_response_chars: usize,
    pub(crate) splash_start: Instant,
    pub(crate) splash_progress: f32,
    pub(crate) ui_opacity: f32,
    pub(crate) transition_to: Option<Screen>,
    pub(crate) transition_start: Option<Instant>,
    pub(crate) transition_phase: Option<TransitionPhase>,
    pub(crate) loading_progress: f32,
    pub(crate) loading_start: Option<Instant>,
    pub(crate) tip_header: String,
    pub(crate) tip_body: String,
    pub(crate) current_tab: usize,
}

impl SpaApp {
    pub(crate) fn new_with_screen(screen: Screen) -> Result<Self, String> {
        if !matches!(
            screen,
            Screen::Splash | Screen::Loading | Screen::Home | Screen::Training
        ) {
            return Err(format!("invalid start screen for new session: {screen:?}"));
        }
        Ok(Self::new_training_phase(screen))
    }

    #[allow(dead_code)]
    pub(crate) fn new() -> Self {
        Self::new_with_screen(Screen::Splash).unwrap()
    }

    fn new_training_phase(screen: Screen) -> Self {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::SystemTime::UNIX_EPOCH)
            .map(|d| d.subsec_nanos())
            .unwrap_or(0);
        let tip = loading_tip(nanos);
        Self {
            session: GameSession::new(1),
            screen,
            focused_action: 0,
            shown_court_logs: 0,
            last_court_log: Instant::now(),
            input: String::new(),
            visible_response_chars: 0,
            splash_start: Instant::now(),
            splash_progress: 0.0,
            ui_opacity: 1.0,
            transition_to: None,
            transition_start: None,
            transition_phase: None,
            loading_progress: 0.0,
            loading_start: matches!(screen, Screen::Loading).then(Instant::now),
            tip_header: tip.0.to_string(),
            tip_body: tip.1.to_string(),
            current_tab: 2, // Default focus on '홈'
        }
    }

    pub(crate) fn screen(&self) -> Screen {
        self.screen
    }

    pub(crate) fn elapsed_splash_progress(&self) -> f32 {
        self.splash_progress.max(
            (self.splash_start.elapsed().as_secs_f32() / SPLASH_PROGRESS_DURATION.as_secs_f32())
                .min(1.0)
                * 100.0,
        )
    }

    pub(crate) fn elapsed_loading_progress(&self) -> f32 {
        self.loading_start
            .map(|start| {
                let t = start.elapsed().as_secs_f32() / LOADING_PROGRESS_DURATION.as_secs_f32();
                ease_out(t.min(1.0)) * 100.0
            })
            .unwrap_or(self.loading_progress)
    }

    #[cfg(test)]
    pub(crate) fn phase(&self) -> GamePhase {
        self.session.phase()
    }

    #[cfg(test)]
    pub(crate) fn court_log_len(&self) -> usize {
        self.session.court_log().len()
    }

    #[cfg(test)]
    pub(crate) fn splash_progress(&self) -> f32 {
        self.splash_progress
    }

    #[cfg(test)]
    pub(crate) fn loading_progress(&self) -> f32 {
        self.loading_progress
    }

    #[cfg(test)]
    pub(crate) fn focused_action(&self) -> usize {
        self.focused_action
    }

    #[cfg(test)]
    pub(crate) fn shown_court_logs(&self) -> usize {
        self.shown_court_logs
    }

    pub(crate) fn input(&self) -> &str {
        &self.input
    }

    pub(crate) fn tick(&mut self) {
        if let Some(target) = self.transition_to {
            let (Some(start), Some(phase)) = (self.transition_start, self.transition_phase) else {
                self.transition_to = None;
                self.transition_start = None;
                self.transition_phase = None;
                self.ui_opacity = 1.0;
                return;
            };
            let elapsed = start.elapsed().as_secs_f64();
            match phase {
                TransitionPhase::FadeOut => {
                    let t = (elapsed / 0.8).clamp(0.0, 1.0) as f32;
                    self.ui_opacity = 1.0 - ease_out(t);
                    if elapsed >= 0.8 {
                        self.screen = target;
                        self.ui_opacity = 0.0;
                        self.transition_start = Some(Instant::now());
                        self.transition_phase = Some(TransitionPhase::FadeIn);
                        if target == Screen::Loading {
                            self.loading_start = Some(Instant::now());
                        }
                    }
                }
                TransitionPhase::FadeIn => {
                    let t = (elapsed / 0.5).clamp(0.0, 1.0) as f32;
                    self.ui_opacity = ease_out(t);
                    if elapsed >= 0.5 {
                        self.transition_to = None;
                        self.transition_phase = None;
                        self.ui_opacity = 1.0;
                    }
                }
            }
        }

        match self.screen {
            Screen::Splash => {
                self.splash_progress = self.elapsed_splash_progress();
            }
            Screen::Loading if self.transition_to.is_none() => {
                self.loading_start.get_or_insert_with(Instant::now);
                self.loading_progress = self.elapsed_loading_progress();
                if self.loading_progress >= 100.0 {
                    self.transition_to = Some(Screen::Home);
                    self.transition_phase = Some(TransitionPhase::FadeOut);
                    self.transition_start = Some(Instant::now());
                    self.loading_start = None;
                }
            }
            Screen::CourtReplay if self.last_court_log.elapsed() >= COURT_LOG_STEP => {
                self.shown_court_logs =
                    (self.shown_court_logs + 1).min(self.session.court_log().len());
                self.last_court_log = Instant::now();
                if self.shown_court_logs == self.session.court_log().len() {
                    self.screen = Screen::Dating;
                }
            }
            Screen::Dating => {
                self.visible_response_chars =
                    (self.visible_response_chars + 1).min(FAKE_RESPONSE.chars().count());
            }
            _ => {}
        }
    }

    pub(crate) fn on_key(&mut self, code: KeyCode) -> Result<bool, String> {
        match (&self.screen, code) {
            // General Quit handling from Splash or Home
            (Screen::Splash | Screen::Home, KeyCode::Esc | KeyCode::Char('q')) => return Ok(true),

            // Esc behavior: transition back to Screen::Home from anywhere else
            (s, KeyCode::Esc) if *s != Screen::Splash => {
                if self.transition_to.is_none() {
                    self.transition_to = Some(Screen::Home);
                    self.transition_phase = Some(TransitionPhase::FadeOut);
                    self.transition_start = Some(Instant::now());
                }
            }

            (Screen::Splash, KeyCode::Enter) => {
                self.screen = Screen::Home;
                self.ui_opacity = 1.0;
                self.transition_to = None;
                self.transition_phase = None;
                self.transition_start = None;
            }

            // Home navigation handling
            (Screen::Home, KeyCode::Left) => {
                self.current_tab = self.current_tab.saturating_sub(1);
            }
            (Screen::Home, KeyCode::Right) => {
                self.current_tab = (self.current_tab + 1).min(4);
            }
            (Screen::Home, KeyCode::Enter) => {
                if self.transition_to.is_none() {
                    let target = match self.current_tab {
                        0 => Some(Screen::Training),
                        1 => Some(Screen::Dating),
                        3 => Some(Screen::CourtReplay),
                        4 => Some(Screen::Result),
                        _ => None,
                    };
                    if let Some(screen) = target {
                        self.transition_to = Some(screen);
                        self.transition_phase = Some(TransitionPhase::FadeOut);
                        self.transition_start = Some(Instant::now());
                    }
                }
            }

            (Screen::Training, KeyCode::Up) => {
                self.focused_action = self.focused_action.saturating_sub(1);
            }
            (Screen::Training, KeyCode::Down) => {
                self.focused_action = (self.focused_action + 1).min(TRAINING_ACTIONS.len() - 1);
            }
            (Screen::Training, KeyCode::Enter) => {
                let action = TRAINING_ACTIONS[self.focused_action];
                self.session
                    .complete_training_action(action.id)
                    .map_err(|error| format!("{error:?}"))?;
                self.screen = Screen::CourtReplay;
                self.last_court_log = Instant::now() - COURT_LOG_STEP;
            }
            (Screen::CourtReplay, KeyCode::Enter) => {
                self.shown_court_logs = self.session.court_log().len();
                self.screen = Screen::Dating;
            }
            (Screen::Dating, KeyCode::Enter) => {
                let input = std::mem::take(&mut self.input);
                self.session
                    .submit_dating_input(input)
                    .map_err(|error| format!("{error:?}"))?;
                self.session
                    .finish_dating(DatingEndReason::Completed)
                    .map_err(|error| format!("{error:?}"))?;
                self.screen = Screen::Result;
            }
            (Screen::Dating, KeyCode::Backspace) => {
                self.input.pop();
            }
            (Screen::Dating, KeyCode::Char(ch)) => {
                self.input.push(ch);
            }
            (Screen::Result, KeyCode::Enter) => return Ok(true),
            _ => {}
        }
        Ok(false)
    }
}
