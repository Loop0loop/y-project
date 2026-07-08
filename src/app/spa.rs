use std::time::{Duration, Instant};

use crossterm::event::KeyCode;

use crate::domain::{
    AdvocateStats, CourtResult, DatingEndReason, DomainCommand, GameSession, TRAINING_ACTIONS,
    phase::GamePhase,
};
use crate::easing::ease_out;

const COURT_LOG_STEP: Duration = Duration::from_millis(520);
pub(crate) const FAKE_RESPONSE: &str =
    "흠, 승부가 완벽했다고는 못 하겠지만... 네 논리에는 확실히 반짝이는 부분이 있었어.";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Screen {
    Splash,
    Loading,
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

pub(crate) struct SpaApp {
    session: GameSession,
    screen: Screen,
    focused_action: usize,
    shown_court_logs: usize,
    last_court_log: Instant,
    input: String,
    visible_response_chars: usize,
    splash_progress: u32,
    ui_opacity: f32,
    transition_to: Option<Screen>,
    transition_start: Option<Instant>,
    transition_phase: Option<TransitionPhase>,
    loading_progress: u32,
    loading_start: Option<Instant>,
    tip_header: String,
    tip_body: String,
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
    pub(crate) ui_opacity: String, // Keep it as string representation for formatting in SVGs
}

impl SpaApp {
    #[allow(dead_code)]
    pub(crate) fn new() -> Self {
        Self::new_training_phase(Screen::Splash)
    }

    pub(crate) fn new_with_screen(screen: Screen) -> Result<Self, String> {
        if !matches!(screen, Screen::Splash | Screen::Loading | Screen::Training) {
            return Err(format!("invalid start screen for new session: {screen:?}"));
        }
        Ok(Self::new_training_phase(screen))
    }

    fn new_training_phase(screen: Screen) -> Self {
        let tips = [
            (
                "휴식 팁",
                "휴식은 전략입니다. 체력이 떨어지면 변론의 타격감이 줄어요.",
            ),
            (
                "변론 팁",
                "상대의 모순을 발견하면 과감하게 '이의있소!'를 외치세요.",
            ),
            (
                "훈련 팁",
                "주차별 일정을 계획하여 능력치를 골고루 성장시켜야 합니다.",
            ),
            (
                "Fontaine 법률",
                "모든 공판은 물의 신 푸리나 님의 참관 하에 집행됩니다.",
            ),
        ];
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::SystemTime::UNIX_EPOCH)
            .map(|d| d.subsec_nanos())
            .unwrap_or(0);
        let idx = (nanos as usize) % tips.len();
        Self {
            session: GameSession::new(1),
            screen,
            focused_action: 0,
            shown_court_logs: 0,
            last_court_log: Instant::now(),
            input: String::new(),
            visible_response_chars: 0,
            splash_progress: 0,
            ui_opacity: 1.0,
            transition_to: None,
            transition_start: None,
            transition_phase: None,
            loading_progress: 0,
            loading_start: None,
            tip_header: tips[idx].0.to_string(),
            tip_body: tips[idx].1.to_string(),
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
                ui_opacity: format!("{:.2}", self.ui_opacity),
            },
            Screen::Loading => AppViewModel {
                phase_label: "LOADING".to_string(),
                title: "LOADING".to_string(),
                subtitle: self.tip_header.clone(),
                body: self.tip_body.clone(),
                side_title: "".to_string(),
                side_body: "".to_string(),
                progress: self.loading_progress,
                screen: Screen::Loading,
                stats: self.session.stats(),
                week: self.session.week(),
                focused_action: self.focused_action,
                ally_hp: self.session.ally_hp(),
                enemy_hp: self.session.enemy_hp(),
                momentum: self.session.momentum(),
                ui_opacity: format!("{:.2}", self.ui_opacity),
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
                    ui_opacity: format!("{:.2}", self.ui_opacity),
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
                ui_opacity: format!("{:.2}", self.ui_opacity),
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
                ui_opacity: format!("{:.2}", self.ui_opacity),
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
                ui_opacity: format!("{:.2}", self.ui_opacity),
            },
        }
    }

    pub(crate) fn screen(&self) -> Screen {
        self.screen
    }

    pub(crate) fn phase(&self) -> GamePhase {
        self.session.phase()
    }

    pub(crate) fn court_log_len(&self) -> usize {
        self.session.court_log().len()
    }

    pub(crate) fn stats(&self) -> AdvocateStats {
        self.session.stats()
    }

    pub(crate) fn court_log(&self) -> &[String] {
        self.session.court_log()
    }

    pub(crate) fn court_result(&self) -> Option<CourtResult> {
        self.session.court_result()
    }

    pub(crate) fn transcript_len(&self) -> usize {
        self.session.transcript_len()
    }

    pub(crate) fn splash_progress(&self) -> u32 {
        self.splash_progress
    }

    pub(crate) fn loading_progress(&self) -> u32 {
        self.loading_progress
    }

    pub(crate) fn loading_tip(&self) -> (&str, &str) {
        (&self.tip_header, &self.tip_body)
    }

    pub(crate) fn focused_action(&self) -> usize {
        self.focused_action
    }

    pub(crate) fn shown_court_logs(&self) -> usize {
        self.shown_court_logs
    }

    pub(crate) fn input(&self) -> &str {
        &self.input
    }

    pub(crate) fn lifecycle_is_valid(&self) -> bool {
        matches!(
            (self.screen, self.session.phase()),
            (
                Screen::Splash | Screen::Loading | Screen::Training,
                GamePhase::Training
            ) | (Screen::CourtReplay | Screen::Dating, GamePhase::Dating)
                | (Screen::Result, GamePhase::Result)
        )
    }

    pub(crate) fn tick(&mut self) {
        // Process general transition fade logic
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
                if self.splash_progress < 100 {
                    self.splash_progress += 4;
                    if self.splash_progress >= 100 {
                        self.splash_progress = 100;
                    }
                }
            }
            Screen::Loading if self.transition_to.is_none() => {
                let elapsed = self
                    .loading_start
                    .get_or_insert_with(Instant::now)
                    .elapsed()
                    .as_secs_f64();
                self.loading_progress = ((elapsed / 4.0) * 100.0).min(100.0) as u32;
                if self.loading_progress >= 100 {
                    self.transition_to = Some(Screen::Training);
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
            (_, KeyCode::Esc | KeyCode::Char('q')) => return Ok(true),
            (Screen::Splash, KeyCode::Enter) => {
                if self.splash_progress < 100 {
                    self.splash_progress = 100;
                } else if self.transition_to.is_none() {
                    self.transition_to = Some(Screen::Loading);
                    self.transition_phase = Some(TransitionPhase::FadeOut);
                    self.transition_start = Some(Instant::now());
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
                    .map_err(|error| format!("{error:?}"))?;
                self.session
                    .apply(DomainCommand::StartCourt)
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
                    .apply(DomainCommand::SubmitDatingInput(input))
                    .map_err(|error| format!("{error:?}"))?;
                self.session
                    .apply(DomainCommand::FinishDating(DatingEndReason::Completed))
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
