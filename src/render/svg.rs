use std::{
    io::{self, Write},
    thread,
    time::Duration,
};

use crate::{
    app::Screen,
    terminal::{
        kitty::{delete_image, present_rgba, KittyImage},
        layout::{rect_to_pixels, CellRect},
        metrics::probe_terminal,
    },
};

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
            subtitle: "fallback svg",
            body: "SVG asset imports were removed",
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
        },
    )?;
    present_demo_rgba(rgba, pixel_width, pixel_height, cell_rect, 424_243, 8)
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
    present_demo_rgba(rgba, pixel_width, pixel_height, cell_rect, 424_244, 9)
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
        },
    )
}

pub(crate) fn render_svg_panel(
    width: u32,
    height: u32,
    spec: PanelSpec<'_>,
) -> Result<Vec<u8>, String> {
    render_svg(width, height, &build_panel_svg(width, height, spec))
}

fn present_demo_rgba(
    rgba: Vec<u8>,
    pixel_width: u32,
    pixel_height: u32,
    cell_rect: CellRect,
    image_id: u32,
    placement_id: u32,
) -> Result<(), String> {
    let image = KittyImage {
        image_id,
        placement_id,
    };
    let mut stdout = io::stdout();
    write!(stdout, "\x1b[?25l").map_err(|error| error.to_string())?;
    present_rgba(
        &mut stdout,
        &rgba,
        pixel_width,
        pixel_height,
        cell_rect,
        &image,
        true,
    )?;
    stdout.flush().map_err(|error| error.to_string())?;
    thread::sleep(Duration::from_secs(3));
    delete_image(&mut stdout, &image)?;
    write!(stdout, "\x1b[?25h\n").map_err(|error| error.to_string())?;
    stdout.flush().map_err(|error| error.to_string())
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
    let progress = spec.bar_percent.min(100);
    let progress_width = progress * 8;
    let phase = escape_xml(spec.phase_label);
    let title = escape_xml(spec.title);
    let subtitle = escape_xml(spec.subtitle);
    let body = escape_xml(&clip_chars(spec.body, 120));
    let side_title = escape_xml(spec.side_title);
    let side_body = escape_xml(&clip_chars(spec.side_body, 120));
    let screen = format!("{:?}", spec.screen);
    format!(
        r##"<svg width="{0}" height="{1}" viewBox="0 0 960 540" xmlns="http://www.w3.org/2000/svg">
<rect width="960" height="540" fill="#05070d"/>
<rect x="32" y="32" width="896" height="476" rx="18" fill="#111827" stroke="#7dd3fc" stroke-width="2"/>
<text x="64" y="78" fill="#7dd3fc" font-family="monospace" font-size="18">{2}</text>
<text x="64" y="138" fill="#ffffff" font-family="monospace" font-size="54" font-weight="700">{3}</text>
<text x="66" y="178" fill="#cbd5e1" font-family="monospace" font-size="22">{4}</text>
<text x="66" y="246" fill="#e5e7eb" font-family="monospace" font-size="22">{5}</text>
<text x="66" y="312" fill="#93c5fd" font-family="monospace" font-size="18">{6}: {7}</text>
<text x="66" y="368" fill="#94a3b8" font-family="monospace" font-size="16">screen={8} week={9} focus={10} hp={11}/{12} momentum={13}</text>
<text x="66" y="398" fill="#94a3b8" font-family="monospace" font-size="16">stats L{14} M{15} S{16} G{17} I{18}</text>
<rect x="64" y="438" width="800" height="24" rx="12" fill="#020617" stroke="#334155"/>
<rect x="64" y="438" width="{19}" height="24" rx="12" fill="#38bdf8"/>
<text x="886" y="457" fill="#e5e7eb" font-family="monospace" font-size="18" text-anchor="end">{20}%</text>
</svg>"##,
        width,
        height,
        phase,
        title,
        subtitle,
        body,
        side_title,
        side_body,
        screen,
        spec.week,
        spec.focused_action,
        spec.ally_hp,
        spec.enemy_hp,
        spec.momentum,
        spec.stats.logic_speed,
        spec.stats.mental_stamina,
        spec.stats.speech_power,
        spec.stats.guts,
        spec.stats.intellect,
        progress_width,
        progress
    )
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
}
