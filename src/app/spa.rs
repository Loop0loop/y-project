use std::time::{Duration, Instant};

use crossterm::event::KeyCode;

use crate::domain::{
    AdvocateStats, CourtResult, DatingEndReason, DomainCommand, GameSession, TRAINING_ACTIONS,
};

const COURT_LOG_STEP: Duration = Duration::from_millis(520);
pub(crate) const FAKE_RESPONSE: &str =
    "흠, 승부가 완벽했다고는 못 하겠지만... 네 논리에는 확실히 반짝이는 부분이 있었어.";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Screen {
    Splash,
    Training,
    CourtReplay,
    Dating,
    Result,
}

pub(crate) struct SpaApp {
    pub(crate) session: GameSession,
    pub(crate) screen: Screen,
    pub(crate) focused_action: usize,
    pub(crate) shown_court_logs: usize,
    last_court_log: Instant,
    pub(crate) input: String,
    visible_response_chars: usize,
    pub(crate) splash_progress: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct AppViewModel {
    pub(crate) phase_label: String,
    pub(crate) title: String,
    pub(crate) subtitle: String,
    pub(crate) body: String,
    pub(crate) side_title: String,
    pub(crate) side_body: String,
    pub(crate) progress: u32,
    pub(crate) screen: Screen,
    pub(crate) stats: AdvocateStats,
    pub(crate) week: u16,
    pub(crate) focused_action: usize,
    pub(crate) ally_hp: i16,
    pub(crate) enemy_hp: i16,
    pub(crate) momentum: i16,
}

impl SpaApp {
    pub(crate) fn new() -> Self {
        Self {
            session: GameSession::new(1),
            screen: Screen::Splash,
            focused_action: 0,
            shown_court_logs: 0,
            last_court_log: Instant::now(),
            input: String::new(),
            visible_response_chars: 0,
            splash_progress: 0,
        }
    }

    pub(crate) fn view_model(&self) -> AppViewModel {
        match self.screen {
            Screen::Splash => AppViewModel {
                phase_label: "INITIALIZING".to_string(),
                title: "BOOT".to_string(),
                subtitle: "advocacy engine".to_string(),
                body: "".to_string(),
                side_title: "".to_string(),
                side_body: "".to_string(),
                progress: self.splash_progress,
                screen: Screen::Splash,
                stats: self.session.stats(),
                week: self.session.week(),
                focused_action: self.focused_action,
                ally_hp: self.session.ally_hp(),
                enemy_hp: self.session.enemy_hp(),
                momentum: self.session.momentum(),
            },
            Screen::Training => {
                let action = TRAINING_ACTIONS[self.focused_action];
                AppViewModel {
                    phase_label: "TRAINING".to_string(),
                    title: "TRAINING".to_string(),
                    subtitle: format!("focus: {}", action.label),
                    body: stats_summary(self.session.stats()),
                    side_title: "ACTIONS".to_string(),
                    side_body: TRAINING_ACTIONS
                        .iter()
                        .enumerate()
                        .map(|(index, action)| {
                            if index == self.focused_action {
                                format!("> {}", action.label)
                            } else {
                                action.label.to_string()
                            }
                        })
                        .collect::<Vec<_>>()
                        .join(" / "),
                    progress: self.focused_action as u32 * 25 + 25,
                    screen: Screen::Training,
                    stats: self.session.stats(),
                    week: self.session.week(),
                    focused_action: self.focused_action,
                    ally_hp: self.session.ally_hp(),
                    enemy_hp: self.session.enemy_hp(),
                    momentum: self.session.momentum(),
                }
            }
            Screen::CourtReplay => AppViewModel {
                phase_label: "COURT".to_string(),
                title: "COURT".to_string(),
                subtitle: format!(
                    "logs {}/{}",
                    self.shown_court_logs,
                    self.session.court_log().len()
                ),
                body: self
                    .session
                    .court_log()
                    .iter()
                    .take(self.shown_court_logs)
                    .last()
                    .cloned()
                    .unwrap_or_else(|| "preparing argument".to_string()),
                side_title: "LOG".to_string(),
                side_body: self
                    .session
                    .court_log()
                    .iter()
                    .take(self.shown_court_logs)
                    .cloned()
                    .collect::<Vec<_>>()
                    .join(" | "),
                progress: percent(self.shown_court_logs, self.session.court_log().len()),
                screen: Screen::CourtReplay,
                stats: self.session.stats(),
                week: self.session.week(),
                focused_action: self.focused_action,
                ally_hp: self.session.ally_hp(),
                enemy_hp: self.session.enemy_hp(),
                momentum: self.session.momentum(),
            },
            Screen::Dating => AppViewModel {
                phase_label: "DATING".to_string(),
                title: "DATING".to_string(),
                subtitle: "fake LLM stream".to_string(),
                body: FAKE_RESPONSE
                    .chars()
                    .take(self.visible_response_chars)
                    .collect::<String>(),
                side_title: "INPUT".to_string(),
                side_body: self.input.clone(),
                progress: percent(self.visible_response_chars, FAKE_RESPONSE.chars().count()),
                screen: Screen::Dating,
                stats: self.session.stats(),
                week: self.session.week(),
                focused_action: self.focused_action,
                ally_hp: self.session.ally_hp(),
                enemy_hp: self.session.enemy_hp(),
                momentum: self.session.momentum(),
            },
            Screen::Result => AppViewModel {
                phase_label: "RESULT".to_string(),
                title: "RESULT".to_string(),
                subtitle: court_result_summary(self.session.court_result()),
                body: format!("{}", self.session.transcript_len()),
                side_title: "SESSION".to_string(),
                side_body: "MVP loop complete".to_string(),
                progress: 100,
                screen: Screen::Result,
                stats: self.session.stats(),
                week: self.session.week(),
                focused_action: self.focused_action,
                ally_hp: self.session.ally_hp(),
                enemy_hp: self.session.enemy_hp(),
                momentum: self.session.momentum(),
            },
        }
    }

    pub(crate) fn tick(&mut self) {
        match self.screen {
            Screen::Splash => {
                if self.splash_progress < 100 {
                    self.splash_progress += 4;
                    if self.splash_progress >= 100 {
                        self.splash_progress = 100;
                    }
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

    pub(crate) fn on_key(&mut self, code: KeyCode) -> bool {
        match (&self.screen, code) {
            (_, KeyCode::Esc | KeyCode::Char('q')) => return true,
            (Screen::Splash, KeyCode::Enter | KeyCode::Down | KeyCode::Char(' ')) => {
                if self.splash_progress < 100 {
                    self.splash_progress = 100;
                } else {
                    self.screen = Screen::Training;
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
                    .apply(DomainCommand::SelectTrainingAction(action.id))
                    .expect("training action");
                self.session
                    .apply(DomainCommand::StartCourt)
                    .expect("court simulation");
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
                    .apply(DomainCommand::SubmitDatingInput(input))
                    .expect("dating input");
                self.session
                    .apply(DomainCommand::FinishDating(DatingEndReason::Completed))
                    .expect("finish dating");
                self.screen = Screen::Result;
            }
            (Screen::Dating, KeyCode::Backspace) => {
                self.input.pop();
            }
            (Screen::Dating, KeyCode::Char(ch)) => {
                self.input.push(ch);
            }
            (Screen::Result, KeyCode::Enter) => return true,
            _ => {}
        }
        false
    }

    pub(crate) fn visible_response_chars(&self) -> usize {
        self.visible_response_chars
    }
}

fn stats_summary(stats: AdvocateStats) -> String {
    format!(
        "LOG {} MEN {} SPC {} GUT {} INT {}",
        stats.logic_speed, stats.mental_stamina, stats.speech_power, stats.guts, stats.intellect
    )
}

fn court_result_summary(result: Option<CourtResult>) -> String {
    result
        .map(|result| format!("{:?} momentum {}", result.verdict, result.final_momentum))
        .unwrap_or_else(|| "no result".to_string())
}

fn percent(current: usize, total: usize) -> u32 {
    if total == 0 {
        0
    } else {
        ((current.min(total) * 100) / total) as u32
    }
}
