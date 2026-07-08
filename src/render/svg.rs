use std::{io::Write, time::Duration};

use crate::{
    app::Screen,
    terminal::{
        kitty::{KittyImage, present_rgba},
        layout::{CellRect, rect_to_pixels},
        metrics::probe_terminal,
        session::{TerminalSession, wait_or_interrupt},
    },
};

const _PANEL_TEMPLATE: &str = include_str!("../../assets/svg/panel.svg");
const SPLASH_TEMPLATE: &str = include_str!("../../assets/svg/splash.svg");
const LOADING_TEMPLATE: &str = include_str!("../../assets/svg/loading.svg");
const TRAINING_TEMPLATE: &str = include_str!("../../assets/svg/training.svg");
const COURT_TEMPLATE: &str = include_str!("../../assets/svg/court.svg");
const DATING_TEMPLATE: &str = include_str!("../../assets/svg/dating.svg");
const RESULT_TEMPLATE: &str = include_str!("../../assets/svg/result.svg");

pub(crate) struct PanelSpec<'a> {
    pub(crate) phase_label: &'a str,
    pub(crate) title: &'a str,
    pub(crate) subtitle: &'a str,
    pub(crate) body: &'a str,
    pub(crate) side_title: &'a str,
    pub(crate) side_body: &'a str,
    pub(crate) bar_percent: u32,
    pub(crate) screen: Screen,
    pub(crate) stats: crate::domain::AdvocateStats,
    pub(crate) week: u16,
    pub(crate) focused_action: usize,
    pub(crate) ally_hp: i16,
    pub(crate) enemy_hp: i16,
    pub(crate) momentum: i16,
    pub(crate) ui_opacity: &'a str,
}

pub(crate) fn run_svg_demo() -> Result<(), String> {
    let metrics = probe_terminal();
    let grid = metrics.grid.ok_or("terminal grid is unknown")?;
    let pixels = metrics.pixels.ok_or("terminal pixel size is unknown")?;
    let cell_rect = CellRect {
        x: 2,
        y: 2,
        width: 40.min(grid.cols.saturating_sub(4)),
        height: 10.min(grid.rows.saturating_sub(4)),
    };
    let (_, _, pixel_width, pixel_height) = rect_to_pixels(grid, pixels, cell_rect);
    let rgba = render_svg_panel(
        pixel_width,
        pixel_height,
        PanelSpec {
            phase_label: "TRAINING",
            title: "TRAINING",
            subtitle: "Logic Speed",
            body: "SVG panel rendered from terminal pixel bbox",
            side_title: "ACTIONS",
            side_body: "Logic / Speech / Law / Nerve",
            bar_percent: 68,
            screen: Screen::Training,
            stats: crate::domain::AdvocateStats::default(),
            week: 1,
            focused_action: 0,
            ally_hp: 100,
            enemy_hp: 100,
            momentum: 0,
            ui_opacity: "1.00",
        },
    )?;
    let image = KittyImage {
        image_id: 424_243,
        placement_id: 8,
    };
    let mut terminal = TerminalSession::enter(false, false)?;
    terminal.register_image(image);

    present_rgba(
        terminal.stdout(),
        &rgba,
        pixel_width,
        pixel_height,
        cell_rect,
        &image,
        true,
    )?;
    write!(
        terminal.stdout(),
        "\x1b[{};{}Hsvg panel bbox={}x{}px cells={}x{} image_id={} placement_id={}   ",
        cell_rect.y + cell_rect.height + 2,
        cell_rect.x + 1,
        pixel_width,
        pixel_height,
        cell_rect.width,
        cell_rect.height,
        image.image_id,
        image.placement_id
    )
    .map_err(|error| error.to_string())?;
    terminal
        .stdout()
        .flush()
        .map_err(|error| error.to_string())?;

    wait_or_interrupt(Duration::from_secs(3))?;
    Ok(())
}

pub(crate) fn run_splash_demo() -> Result<(), String> {
    let metrics = probe_terminal();
    let grid = metrics.grid.ok_or("terminal grid is unknown")?;
    let pixels = metrics.pixels.ok_or("terminal pixel size is unknown")?;
    let cell_rect = CellRect {
        x: 0,
        y: 0,
        width: grid.cols,
        height: grid.rows,
    };
    let (_, _, pixel_width, pixel_height) = rect_to_pixels(grid, pixels, cell_rect);
    let rgba = render_splash(pixel_width, pixel_height, 68)?;
    let image = KittyImage {
        image_id: 424_244,
        placement_id: 9,
    };
    let mut terminal = TerminalSession::enter(false, false)?;
    terminal.register_image(image);

    present_rgba(
        terminal.stdout(),
        &rgba,
        pixel_width,
        pixel_height,
        cell_rect,
        &image,
        true,
    )?;
    terminal
        .stdout()
        .flush()
        .map_err(|error| error.to_string())?;

    wait_or_interrupt(Duration::from_secs(3))?;
    Ok(())
}

pub(crate) fn render_splash(width: u32, height: u32, progress: u32) -> Result<Vec<u8>, String> {
    render_svg_panel(
        width,
        height,
        PanelSpec {
            phase_label: "INITIALIZING",
            title: "BOOT",
            subtitle: "advocacy engine",
            body: "",
            side_title: "",
            side_body: "",
            bar_percent: progress,
            screen: Screen::Splash,
            stats: crate::domain::AdvocateStats::default(),
            week: 1,
            focused_action: 0,
            ally_hp: 100,
            enemy_hp: 100,
            momentum: 0,
            ui_opacity: "1.00",
        },
    )
}

pub(crate) fn render_svg_panel(
    width: u32,
    height: u32,
    spec: PanelSpec<'_>,
) -> Result<Vec<u8>, String> {
    let svg = build_panel_svg(width, height, spec);
    render_svg(width, height, &svg)
}

fn render_svg(width: u32, height: u32, svg: &str) -> Result<Vec<u8>, String> {
    let mut options = resvg::usvg::Options::default();
    options.fontdb_mut().load_system_fonts();
    let tree = resvg::usvg::Tree::from_str(svg, &options).map_err(|error| error.to_string())?;
    let mut pixmap =
        resvg::tiny_skia::Pixmap::new(width, height).ok_or("failed to allocate SVG pixmap")?;
    resvg::render(
        &tree,
        resvg::tiny_skia::Transform::identity(),
        &mut pixmap.as_mut(),
    );
    Ok(pixmap.data().to_vec())
}

fn build_panel_svg(width: u32, height: u32, spec: PanelSpec<'_>) -> String {
    let template = match spec.screen {
        Screen::Splash => SPLASH_TEMPLATE,
        Screen::Loading => LOADING_TEMPLATE,
        Screen::Training => TRAINING_TEMPLATE,
        Screen::CourtReplay => COURT_TEMPLATE,
        Screen::Dating => DATING_TEMPLATE,
        Screen::Result => RESULT_TEMPLATE,
    };

    let phase_label = escape_xml(spec.phase_label);
    let title = escape_xml(spec.title);
    let subtitle = escape_xml(spec.subtitle);
    let body = escape_xml(&clip_chars(spec.body, 120));
    let side_title = escape_xml(spec.side_title);
    let side_body = escape_xml(&clip_chars(spec.side_body, 120));
    let progress_label = format!("{}%", spec.bar_percent.min(100));

    let mut svg = template
        .replace("{{WIDTH}}", &width.to_string())
        .replace("{{HEIGHT}}", &height.to_string())
        .replace("{{UI_OPACITY}}", spec.ui_opacity)
        .replace("{{PHASE_LABEL}}", &phase_label)
        .replace("{{TITLE}}", &title)
        .replace("{{SUBTITLE}}", &subtitle)
        .replace("{{BODY}}", &body)
        .replace("{{SIDE_TITLE}}", &side_title)
        .replace("{{SIDE_BODY}}", &side_body)
        .replace("{{PROGRESS}}", &spec.bar_percent.min(100).to_string())
        .replace("{{PROGRESS_LABEL}}", &progress_label)
        .replace("{{WEEK}}", &spec.week.to_string());

    // Phase-specific tokens replacement
    match spec.screen {
        Screen::Splash => {
            let pb_width = (500 * spec.bar_percent.min(100)) / 100;
            svg = svg.replace("{{PROGRESS_BAR_WIDTH}}", &pb_width.to_string());
        }
        Screen::Loading => {
            let pb_width = (520 * spec.bar_percent.min(100)) / 100;
            svg = svg.replace("{{PROGRESS_BAR_WIDTH}}", &pb_width.to_string());
        }
        Screen::Training => {
            svg = svg
                .replace("{{STAT_LOGIC}}", &spec.stats.logic_speed.to_string())
                .replace("{{STAT_MENTAL}}", &spec.stats.mental_stamina.to_string())
                .replace("{{STAT_SPEECH}}", &spec.stats.speech_power.to_string())
                .replace("{{STAT_GUTS}}", &spec.stats.guts.to_string())
                .replace("{{STAT_INTELLECT}}", &spec.stats.intellect.to_string())
                .replace(
                    "{{BAR_LOGIC_WIDTH}}",
                    &((340 * spec.stats.logic_speed.min(100)) / 100).to_string(),
                )
                .replace(
                    "{{BAR_MENTAL_WIDTH}}",
                    &((340 * spec.stats.mental_stamina.min(100)) / 100).to_string(),
                )
                .replace(
                    "{{BAR_SPEECH_WIDTH}}",
                    &((340 * spec.stats.speech_power.min(100)) / 100).to_string(),
                )
                .replace(
                    "{{BAR_GUTS_WIDTH}}",
                    &((340 * spec.stats.guts.min(100)) / 100).to_string(),
                )
                .replace(
                    "{{BAR_INTELLECT_WIDTH}}",
                    &((340 * spec.stats.intellect.min(100)) / 100).to_string(),
                );

            for action_idx in 0..4 {
                let opacity = if spec.focused_action == action_idx {
                    "0.85"
                } else {
                    "0.15"
                };
                let border = if spec.focused_action == action_idx {
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
        Screen::CourtReplay => {
            let ally_hp_w = (220 * spec.ally_hp.max(0).min(100) as u32) / 100;
            let enemy_hp_w = (220 * spec.enemy_hp.max(0).min(100) as u32) / 100;

            // Momentum center offset calculation: zero center at 540
            let offset = (spec.momentum.clamp(-100, 100) as i32 * 540) / 100;
            let momentum_x = (540 + offset - 15).max(0).min(1050);

            let objection_opacity = if spec.body.contains("contradiction") {
                "1.0"
            } else {
                "0.0"
            };

            svg = svg
                .replace("{{ALLY_HP}}", &spec.ally_hp.to_string())
                .replace("{{ENEMY_HP}}", &spec.enemy_hp.to_string())
                .replace("{{BAR_ALLY_HP_WIDTH}}", &ally_hp_w.to_string())
                .replace("{{BAR_ENEMY_HP_WIDTH}}", &enemy_hp_w.to_string())
                .replace("{{MOMENTUM}}", &spec.momentum.to_string())
                .replace("{{BAR_MOMENTUM_X}}", &momentum_x.to_string())
                .replace("{{BAR_MOMENTUM_WIDTH}}", "30")
                .replace("{{OBJECTION_OPACITY}}", objection_opacity);
        }
        Screen::Result => {
            svg = svg
                .replace("{{STAT_LOGIC}}", &spec.stats.logic_speed.to_string())
                .replace("{{STAT_MENTAL}}", &spec.stats.mental_stamina.to_string())
                .replace("{{STAT_SPEECH}}", &spec.stats.speech_power.to_string())
                .replace("{{STAT_GUTS}}", &spec.stats.guts.to_string())
                .replace("{{STAT_INTELLECT}}", &spec.stats.intellect.to_string());
        }
        _ => {}
    }

    svg
}

fn escape_xml(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

fn clip_chars(value: &str, limit: usize) -> String {
    let mut clipped: String = value.chars().take(limit).collect();
    if value.chars().count() > limit {
        clipped.push_str("...");
    }
    clipped
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_svg_with_requested_size() {
        let svg = build_panel_svg(
            320,
            140,
            PanelSpec {
                phase_label: "TRAIN",
                title: "A&B",
                subtitle: "S",
                body: "B",
                side_title: "X",
                side_body: "Y",
                bar_percent: 68,
                screen: Screen::Result,
                stats: crate::domain::AdvocateStats::default(),
                week: 1,
                focused_action: 0,
                ally_hp: 100,
                enemy_hp: 100,
                momentum: 0,
                ui_opacity: "1.00",
            },
        );
        assert!(svg.contains(r#"width="320""#));
        assert!(svg.contains(r#"height="140""#));
        assert!(svg.contains("A&amp;B"));
        assert!(!svg.contains("{{"));
    }

    #[test]
    fn renders_svg_panel_to_rgba_buffer() {
        let rgba = render_svg_panel(
            64,
            32,
            PanelSpec {
                phase_label: "T",
                title: "T",
                subtitle: "S",
                body: "B",
                side_title: "X",
                side_body: "Y",
                bar_percent: 68,
                screen: Screen::Training,
                stats: crate::domain::AdvocateStats::default(),
                week: 1,
                focused_action: 0,
                ally_hp: 100,
                enemy_hp: 100,
                momentum: 0,
                ui_opacity: "1.00",
            },
        )
        .expect("render svg");
        assert_eq!(rgba.len(), 64 * 32 * 4);
        assert!(rgba.iter().any(|channel| *channel != 0));
    }

    #[test]
    fn renders_splash_to_rgba_buffer() {
        let rgba = render_splash(160, 90, 68).expect("render splash");
        assert_eq!(rgba.len(), 160 * 90 * 4);
        assert!(rgba.iter().any(|channel| *channel != 0));
    }

    #[test]
    fn all_scene_templates_resolve_tokens() {
        for screen in [
            Screen::Splash,
            Screen::Loading,
            Screen::Training,
            Screen::CourtReplay,
            Screen::Dating,
            Screen::Result,
        ] {
            let svg = build_panel_svg(
                320,
                480,
                PanelSpec {
                    phase_label: "PHASE",
                    title: "TITLE",
                    subtitle: "SUB",
                    body: "contradiction body",
                    side_title: "SIDE",
                    side_body: "SIDE BODY",
                    bar_percent: 68,
                    screen,
                    stats: crate::domain::AdvocateStats::default(),
                    week: 1,
                    focused_action: 0,
                    ally_hp: 100,
                    enemy_hp: 100,
                    momentum: 0,
                    ui_opacity: "1.00",
                },
            );
            assert!(!svg.contains("{{"), "{screen:?} has unresolved token");
        }
    }
}
