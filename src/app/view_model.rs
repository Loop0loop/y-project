use super::{FAKE_RESPONSE, Screen, SpaApp};
use crate::domain::{AdvocateStats, CourtResult, TRAINING_ACTIONS};
use crate::render::RenderView;

impl SpaApp {
    pub(crate) fn view_model(&self) -> RenderView {
        match self.screen {
            Screen::Splash => RenderView {
                scene: self.screen.into(),
                phase_label: "INITIALIZING".to_string(),
                title: "BOOT".to_string(),
                subtitle: "advocacy engine".to_string(),
                body: "".to_string(),
                side_title: "".to_string(),
                side_body: "".to_string(),
                progress: self.splash_progress as f32,
                stats: self.session.stats(),
                week: self.session.week(),
                focused_action: self.focused_action,
                ally_hp: self.session.ally_hp(),
                enemy_hp: self.session.enemy_hp(),
                momentum: self.session.momentum(),
                ui_opacity: format!("{:.2}", self.ui_opacity),
                current_tab: self.current_tab,
            },
            Screen::Loading => RenderView {
                scene: self.screen.into(),
                phase_label: "LOADING".to_string(),
                title: "LOADING".to_string(),
                subtitle: self.tip_header.clone(),
                body: self.tip_body.clone(),
                side_title: "".to_string(),
                side_body: "".to_string(),
                progress: self.loading_progress as f32,
                stats: self.session.stats(),
                week: self.session.week(),
                focused_action: self.focused_action,
                ally_hp: self.session.ally_hp(),
                enemy_hp: self.session.enemy_hp(),
                momentum: self.session.momentum(),
                ui_opacity: format!("{:.2}", self.ui_opacity),
                current_tab: self.current_tab,
            },
            Screen::Home => RenderView {
                scene: self.screen.into(),
                phase_label: "HOME".to_string(),
                title: "HOME".to_string(),
                subtitle: "Palais Mermonia".to_string(),
                body: "법정에서의 승리는, 사전 조사에서 시작됩니다.".to_string(),
                side_title: "".to_string(),
                side_body: "".to_string(),
                progress: 0.0,
                stats: self.session.stats(),
                week: self.session.week(),
                focused_action: self.focused_action,
                ally_hp: self.session.ally_hp(),
                enemy_hp: self.session.enemy_hp(),
                momentum: self.session.momentum(),
                ui_opacity: format!("{:.2}", self.ui_opacity),
                current_tab: self.current_tab,
            },
            Screen::Training => {
                let action = TRAINING_ACTIONS[self.focused_action];
                RenderView {
                    scene: self.screen.into(),
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
                    progress: self.focused_action as f32 * 25.0 + 25.0,
                    stats: self.session.stats(),
                    week: self.session.week(),
                    focused_action: self.focused_action,
                    ally_hp: self.session.ally_hp(),
                    enemy_hp: self.session.enemy_hp(),
                    momentum: self.session.momentum(),
                    ui_opacity: format!("{:.2}", self.ui_opacity),
                    current_tab: self.current_tab,
                }
            }
            Screen::CourtReplay => RenderView {
                scene: self.screen.into(),
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
                stats: self.session.stats(),
                week: self.session.week(),
                focused_action: self.focused_action,
                ally_hp: self.session.ally_hp(),
                enemy_hp: self.session.enemy_hp(),
                momentum: self.session.momentum(),
                ui_opacity: format!("{:.2}", self.ui_opacity),
                current_tab: self.current_tab,
            },
            Screen::Dating => RenderView {
                scene: self.screen.into(),
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
                stats: self.session.stats(),
                week: self.session.week(),
                focused_action: self.focused_action,
                ally_hp: self.session.ally_hp(),
                enemy_hp: self.session.enemy_hp(),
                momentum: self.session.momentum(),
                ui_opacity: format!("{:.2}", self.ui_opacity),
                current_tab: self.current_tab,
            },
            Screen::Result => RenderView {
                scene: self.screen.into(),
                phase_label: "RESULT".to_string(),
                title: "RESULT".to_string(),
                subtitle: self
                    .session
                    .court_result()
                    .map(|r| court_result_summary(r))
                    .unwrap_or_else(|| "no result".to_string()),
                body: format!("{}", self.session.transcript_len()),
                side_title: "SESSION".to_string(),
                side_body: "MVP loop complete".to_string(),
                progress: 100.0,
                stats: self.session.stats(),
                week: self.session.week(),
                focused_action: self.focused_action,
                ally_hp: self.session.ally_hp(),
                enemy_hp: self.session.enemy_hp(),
                momentum: self.session.momentum(),
                ui_opacity: format!("{:.2}", self.ui_opacity),
                current_tab: self.current_tab,
            },
        }
    }
}

fn stats_summary(stats: AdvocateStats) -> String {
    format!(
        "LOG {} MEN {} SPC {} GUT {} INT {}",
        stats.logic_speed, stats.mental_stamina, stats.speech_power, stats.guts, stats.intellect
    )
}

fn court_result_summary(result: CourtResult) -> String {
    format!(
        "verdict={:?} momentum={}",
        result.verdict, result.final_momentum
    )
}

fn percent(value: usize, total: usize) -> f32 {
    if total == 0 {
        0.0
    } else {
        (value as f32 / total as f32) * 100.0
    }
}
