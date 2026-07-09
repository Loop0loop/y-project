use crate::shared::{LOADING_PROGRESS_WIDTH, SPLASH_PROGRESS_WIDTH, escape_xml};

use super::{raster::rasterize_svg, scene::SceneKind, tokens::RenderView};

const SPLASH_TEMPLATE: &str = include_str!("../../assets/svg/splash.svg");
const LOADING_TEMPLATE: &str = include_str!("../../assets/svg/loading.svg");
const HOME_TEMPLATE: &str = include_str!("../../assets/svg/home.svg");
// ponytail: deleted scene SVGs reuse Home until those screens get new assets.
const TRAINING_TEMPLATE: &str = HOME_TEMPLATE;
const COURT_TEMPLATE: &str = HOME_TEMPLATE;
const DATING_TEMPLATE: &str = HOME_TEMPLATE;
const RESULT_TEMPLATE: &str = HOME_TEMPLATE;

pub(crate) fn render_splash(width: u32, height: u32, progress: f32) -> Result<Vec<u8>, String> {
    render_view_rgba(
        width,
        height,
        &RenderView {
            scene: SceneKind::Splash,
            phase_label: "INITIALIZING".to_string(),
            title: "BOOT".to_string(),
            subtitle: "advocacy engine".to_string(),
            body: String::new(),
            side_title: String::new(),
            side_body: String::new(),
            progress,
            stats: crate::domain::AdvocateStats::default(),
            week: 1,
            focused_action: 0,
            ally_hp: 100,
            enemy_hp: 100,
            momentum: 0,
            ui_opacity: "1.00".to_string(),
            current_tab: 2,
        },
    )
}

pub(crate) fn render_view_rgba(
    width: u32,
    height: u32,
    view: &RenderView,
) -> Result<Vec<u8>, String> {
    let svg = build_view_svg(width, height, view);
    rasterize_svg(width, height, &svg)
}

fn build_view_svg(width: u32, height: u32, view: &RenderView) -> String {
    let template = match view.scene {
        SceneKind::Splash => SPLASH_TEMPLATE,
        SceneKind::Loading => LOADING_TEMPLATE,
        SceneKind::Home => HOME_TEMPLATE,
        SceneKind::Training => TRAINING_TEMPLATE,
        SceneKind::Court => COURT_TEMPLATE,
        SceneKind::Dating => DATING_TEMPLATE,
        SceneKind::Result => RESULT_TEMPLATE,
    };

    let phase_label = escape_xml(&view.phase_label);
    let title = escape_xml(&view.title);
    let subtitle = escape_xml(&view.subtitle);
    let body = escape_xml(&clip_chars(&view.body, 120));
    let side_title = escape_xml(&view.side_title);
    let side_body = escape_xml(&clip_chars(&view.side_body, 120));
    let progress = view.progress.clamp(0.0, 100.0);
    let progress_label = format!("{progress:.0}%");

    let mut svg = template
        .replace("{{WIDTH}}", &width.to_string())
        .replace("{{HEIGHT}}", &height.to_string())
        .replace("{{UI_OPACITY}}", &view.ui_opacity)
        .replace("{{PHASE_LABEL}}", &phase_label)
        .replace("{{TITLE}}", &title)
        .replace("{{SUBTITLE}}", &subtitle)
        .replace("{{BODY}}", &body)
        .replace("{{SIDE_TITLE}}", &side_title)
        .replace("{{SIDE_BODY}}", &side_body)
        .replace("{{PROGRESS}}", &format!("{progress:.0}"))
        .replace("{{PROGRESS_LABEL}}", &progress_label)
        .replace("{{WEEK}}", &view.week.to_string());

    // Phase-specific tokens replacement
    match view.scene {
        SceneKind::Splash => {
            svg = svg.replace(
                "{{PROGRESS_BAR_WIDTH}}",
                &format!("{:.2}", SPLASH_PROGRESS_WIDTH * progress / 100.0),
            );
        }
        SceneKind::Loading => {
            svg = svg.replace(
                "{{PROGRESS_BAR_WIDTH}}",
                &format!("{:.2}", LOADING_PROGRESS_WIDTH * progress / 100.0),
            );
        }
        SceneKind::Training => {
            svg = svg
                .replace("{{STAT_LOGIC}}", &view.stats.logic_speed.to_string())
                .replace("{{STAT_MENTAL}}", &view.stats.mental_stamina.to_string())
                .replace("{{STAT_SPEECH}}", &view.stats.speech_power.to_string())
                .replace("{{STAT_GUTS}}", &view.stats.guts.to_string())
                .replace("{{STAT_INTELLECT}}", &view.stats.intellect.to_string())
                .replace(
                    "{{BAR_LOGIC_WIDTH}}",
                    &((340 * view.stats.logic_speed.min(100)) / 100).to_string(),
                )
                .replace(
                    "{{BAR_MENTAL_WIDTH}}",
                    &((340 * view.stats.mental_stamina.min(100)) / 100).to_string(),
                )
                .replace(
                    "{{BAR_SPEECH_WIDTH}}",
                    &((340 * view.stats.speech_power.min(100)) / 100).to_string(),
                )
                .replace(
                    "{{BAR_GUTS_WIDTH}}",
                    &((340 * view.stats.guts.min(100)) / 100).to_string(),
                )
                .replace(
                    "{{BAR_INTELLECT_WIDTH}}",
                    &((340 * view.stats.intellect.min(100)) / 100).to_string(),
                );

            for action_idx in 0..4 {
                let opacity = if view.focused_action == action_idx {
                    "0.85"
                } else {
                    "0.15"
                };
                let border = if view.focused_action == action_idx {
                    "#00d5ff"
                } else {
                    "#24506d"
                };
                svg = svg
                    .replace(
                        &format!("{{{{ACTION_{}_FOCUS_OPACITY}}}}", action_idx),
                        opacity,
                    )
                    .replace(
                        &format!("{{{{ACTION_{}_BORDER_COLOR}}}}", action_idx),
                        border,
                    );
            }
        }
        SceneKind::Court => {
            let ally_hp_w = (220 * view.ally_hp.clamp(0, 100) as u32) / 100;
            let enemy_hp_w = (220 * view.enemy_hp.clamp(0, 100) as u32) / 100;

            // Momentum center offset calculation: zero center at 540
            let offset = (view.momentum.clamp(-100, 100) as i32 * 540) / 100;
            let momentum_x = (540 + offset - 15).clamp(0, 1050);

            let objection_opacity = if view.body.contains("contradiction") {
                "1.0"
            } else {
                "0.0"
            };

            svg = svg
                .replace("{{ALLY_HP}}", &view.ally_hp.to_string())
                .replace("{{ENEMY_HP}}", &view.enemy_hp.to_string())
                .replace("{{BAR_ALLY_HP_WIDTH}}", &ally_hp_w.to_string())
                .replace("{{BAR_ENEMY_HP_WIDTH}}", &enemy_hp_w.to_string())
                .replace("{{MOMENTUM}}", &view.momentum.to_string())
                .replace("{{BAR_MOMENTUM_X}}", &momentum_x.to_string())
                .replace("{{BAR_MOMENTUM_WIDTH}}", "30")
                .replace("{{OBJECTION_OPACITY}}", objection_opacity);
        }
        SceneKind::Result => {
            svg = svg
                .replace("{{STAT_LOGIC}}", &view.stats.logic_speed.to_string())
                .replace("{{STAT_MENTAL}}", &view.stats.mental_stamina.to_string())
                .replace("{{STAT_SPEECH}}", &view.stats.speech_power.to_string())
                .replace("{{STAT_GUTS}}", &view.stats.guts.to_string())
                .replace("{{STAT_INTELLECT}}", &view.stats.intellect.to_string());
        }
        SceneKind::Home | SceneKind::Dating => {}
    }

    let indicator_x = 18.0 + (view.current_tab as f32) * 112.8;
    svg = svg.replace("{{TAB_INDICATOR_X}}", &format!("{:.2}", indicator_x));

    for tab_idx in 0..5 {
        let opacity = if view.current_tab == tab_idx {
            "1.00"
        } else {
            "0.00"
        };
        svg = svg.replace(
            &format!("{{{{TAB_{}_HIGHLIGHT_OPACITY}}}}", tab_idx),
            opacity,
        );
    }

    svg
}

fn clip_chars(value: &str, limit: usize) -> String {
    let mut clipped: String = value.chars().take(limit).collect();
    if value.chars().count() > limit {
        clipped.push_str("...");
    }
    clipped
}

#[cfg(test)]
#[path = "svg_tests.rs"]
mod tests;
