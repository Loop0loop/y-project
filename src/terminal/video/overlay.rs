use std::path::{Path, PathBuf};
use std::time::Instant;

use crate::easing::ease_out;
use crate::terminal::{
    kitty::{KittyImage, present_rgba_with_z},
    layout::{CellRect, rect_to_pixels},
    metrics::{TerminalGrid, TerminalPixels, probe_terminal},
};

#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) enum OverlayPhase {
    SplashActive,
    SplashFadeOut { start: Instant },
    LoadingFadeIn { start: Instant },
    LoadingActive { start: Instant },
    Finished,
}

#[derive(Debug, Clone, PartialEq)]
struct RenderState {
    path: PathBuf,
    render_width: u32,
    render_height: u32,
    progress_key: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct OverlayLayout {
    terminal_cols: u16,
    terminal_rows: u16,
    cell_rect: CellRect,
    render_width: u32,
    render_height: u32,
}

pub(super) struct SplashOverlay {
    rgba: Vec<u8>,
    scratch: Vec<u8>,
    image: KittyImage,
    pub(super) phase: OverlayPhase,
    last_state: Option<RenderState>,
    last_cell_rect: Option<CellRect>,
    last_layout: Option<OverlayLayout>,
    tip_header: String,
    tip_body: String,
    splash_path: PathBuf,
}

struct OverlayFrame {
    opacity: f32,
    progress: f32,
    path: PathBuf,
}

impl SplashOverlay {
    pub(super) fn new(path: PathBuf) -> Self {
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
            rgba: Vec::new(),
            scratch: Vec::new(),
            image: KittyImage {
                image_id: 525_310,
                placement_id: 31,
            },
            phase: OverlayPhase::SplashActive,
            last_state: None,
            last_cell_rect: None,
            last_layout: None,
            tip_header: tips[idx].0.to_string(),
            tip_body: tips[idx].1.to_string(),
            splash_path: path,
        }
    }

    pub(super) fn trigger_start(&mut self) {
        if self.phase == OverlayPhase::SplashActive {
            self.phase = OverlayPhase::SplashFadeOut {
                start: Instant::now(),
            };
        }
    }

    pub(super) fn is_finished(&self) -> bool {
        self.phase == OverlayPhase::Finished
    }

    pub(super) fn image(&self) -> KittyImage {
        self.image
    }

    pub(super) fn present_layer(
        &mut self,
        stdout: &mut impl std::io::Write,
        terminal_cols: u16,
        terminal_rows: u16,
    ) -> Result<(), String> {
        let frame = self.next_frame();
        let layout = self.overlay_layout(terminal_cols, terminal_rows)?;
        let cell_rect = layout.cell_rect;
        let render_width = layout.render_width;
        let render_height = layout.render_height;
        self.ensure_rendered(&frame.path, render_width, render_height, frame.progress)?;
        let image = self.image;
        let rgba = if frame.opacity >= 0.995 {
            self.rgba.as_slice()
        } else {
            self.rgba_with_opacity(frame.opacity)
        };
        present_rgba_with_z(
            stdout,
            rgba,
            render_width,
            render_height,
            cell_rect,
            &image,
            true,
            10,
        )?;
        self.last_cell_rect = Some(cell_rect);
        Ok(())
    }

    fn next_frame(&mut self) -> OverlayFrame {
        loop {
            let (opacity, progress, path) = match self.phase {
                OverlayPhase::SplashActive => (1.0f32, 0.0f32, self.splash_path.clone()),
                OverlayPhase::SplashFadeOut { start } => {
                    let elapsed = start.elapsed().as_secs_f64();
                    if elapsed >= 0.8 {
                        self.phase = OverlayPhase::LoadingFadeIn {
                            start: Instant::now(),
                        };
                        continue;
                    }
                    let t = (elapsed / 0.8) as f32;
                    (1.0 - ease_out(t), 0.0, self.splash_path.clone())
                }
                OverlayPhase::LoadingFadeIn { start } => {
                    let elapsed = start.elapsed().as_secs_f64();
                    if elapsed >= 0.5 {
                        self.phase = OverlayPhase::LoadingActive {
                            start: Instant::now(),
                        };
                        continue;
                    }
                    let t = (elapsed / 0.5) as f32;
                    (ease_out(t), 0.0, PathBuf::from("assets/svg/loading.svg"))
                }
                OverlayPhase::LoadingActive { start } => {
                    let elapsed = start.elapsed().as_secs_f64();
                    let t = (elapsed / 4.0).clamp(0.0, 1.0) as f32;
                    let progress = ease_out(t) * 100.0;
                    if progress >= 100.0 {
                        self.phase = OverlayPhase::Finished;
                    }
                    (1.0, progress, PathBuf::from("assets/svg/loading.svg"))
                }
                OverlayPhase::Finished => (0.0, 100.0, PathBuf::from("assets/svg/loading.svg")),
            };

            return OverlayFrame {
                opacity,
                progress,
                path,
            };
        }
    }

    fn ensure_rendered(
        &mut self,
        path: &Path,
        render_width: u32,
        render_height: u32,
        progress: f32,
    ) -> Result<(), String> {
        let progress_key = (progress * 10.0).round() as u32;
        let state = RenderState {
            path: path.to_path_buf(),
            render_width,
            render_height,
            progress_key,
        };

        if self.last_state.as_ref() == Some(&state) {
            return Ok(());
        }

        let svg = std::fs::read_to_string(path)
            .map_err(|error| format!("failed to read overlay SVG {}: {error}", path.display()))?
            .replace("{{WIDTH}}", &render_width.to_string())
            .replace("{{HEIGHT}}", &render_height.to_string())
            .replace("{{UI_OPACITY}}", "1")
            .replace(
                "{{PROGRESS}}",
                &format!("{:.0}", progress.clamp(0.0, 100.0)),
            )
            .replace(
                "{{PROGRESS_BAR_WIDTH}}",
                &format!("{:.2}", 520.0 * progress.clamp(0.0, 100.0) / 100.0),
            )
            .replace("{{SUBTITLE}}", &escape_xml(&self.tip_header))
            .replace("{{BODY}}", &escape_xml(&self.tip_body));

        let mut options = resvg::usvg::Options::default();
        options.fontdb_mut().load_system_fonts();
        let tree =
            resvg::usvg::Tree::from_str(&svg, &options).map_err(|error| error.to_string())?;
        let mut pixmap = resvg::tiny_skia::Pixmap::new(render_width, render_height)
            .ok_or("failed to allocate overlay pixmap")?;
        resvg::render(
            &tree,
            resvg::tiny_skia::Transform::identity(),
            &mut pixmap.as_mut(),
        );
        self.rgba = pixmap.data().to_vec();
        self.last_state = Some(state);
        Ok(())
    }

    fn rgba_with_opacity(&mut self, opacity: f32) -> &[u8] {
        apply_opacity(&self.rgba, opacity, &mut self.scratch);
        &self.scratch
    }

    fn overlay_layout(
        &mut self,
        terminal_cols: u16,
        terminal_rows: u16,
    ) -> Result<OverlayLayout, String> {
        if let Some(layout) = self.last_layout {
            if layout.terminal_cols == terminal_cols && layout.terminal_rows == terminal_rows {
                return Ok(layout);
            }
        }
        let layout = overlay_layout(terminal_cols, terminal_rows)?;
        self.last_layout = Some(layout);
        Ok(layout)
    }
}

fn overlay_layout(terminal_cols: u16, terminal_rows: u16) -> Result<OverlayLayout, String> {
    let metrics = probe_terminal();
    let grid = metrics.grid.ok_or("terminal grid is unknown")?;
    let pixels = metrics.pixels.ok_or("terminal pixel size is unknown")?;
    let cell_rect = fit_overlay_rect(grid, pixels, terminal_cols, terminal_rows);
    let (_, _, pixel_width, pixel_height) = rect_to_pixels(grid, pixels, cell_rect);
    let (render_width, render_height) = capped_render_size(pixel_width, pixel_height, 600, 900);
    Ok(OverlayLayout {
        terminal_cols,
        terminal_rows,
        cell_rect,
        render_width,
        render_height,
    })
}

fn fit_overlay_rect(
    grid: TerminalGrid,
    pixels: TerminalPixels,
    terminal_cols: u16,
    terminal_rows: u16,
) -> CellRect {
    let cols = terminal_cols.min(grid.cols).max(1);
    let rows = terminal_rows.min(grid.rows).max(1);
    let cell_w = f64::from(pixels.width) / f64::from(grid.cols.max(1));
    let cell_h = f64::from(pixels.height) / f64::from(grid.rows.max(1));
    let available_w = f64::from(cols) * cell_w;
    let available_h = f64::from(rows) * cell_h;
    let target_aspect = 600.0 / 900.0;

    let (pixel_w, pixel_h) = if available_w / available_h.max(1.0) > target_aspect {
        (available_h * target_aspect, available_h)
    } else {
        (available_w, available_w / target_aspect)
    };
    let width = ((pixel_w / cell_w).round() as u16).clamp(1, cols);
    let height = ((pixel_h / cell_h).round() as u16).clamp(1, rows);

    CellRect {
        x: cols.saturating_sub(width) / 2,
        y: rows.saturating_sub(height) / 2,
        width,
        height,
    }
}

fn capped_render_size(width: u32, height: u32, max_w: u32, max_h: u32) -> (u32, u32) {
    if width <= max_w && height <= max_h {
        return (width.max(1), height.max(1));
    }
    let scale = (max_w as f64 / width.max(1) as f64).min(max_h as f64 / height.max(1) as f64);
    (
        ((width as f64 * scale).round() as u32).max(1),
        ((height as f64 * scale).round() as u32).max(1),
    )
}

fn apply_opacity(rgba: &[u8], opacity: f32, out: &mut Vec<u8>) {
    let opacity = opacity.clamp(0.0, 1.0);
    out.clear();
    out.extend_from_slice(rgba);
    for pixel in out.chunks_exact_mut(4) {
        pixel[0] = scale_channel(pixel[0], opacity);
        pixel[1] = scale_channel(pixel[1], opacity);
        pixel[2] = scale_channel(pixel[2], opacity);
        pixel[3] = scale_channel(pixel[3], opacity);
    }
}

fn scale_channel(value: u8, opacity: f32) -> u8 {
    (f32::from(value) * opacity).round() as u8
}

fn escape_xml(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn opacity_scales_rgba_channels() {
        let mut out = Vec::new();
        apply_opacity(&[50, 20, 10, 128], 0.5, &mut out);
        assert_eq!(out, vec![25, 10, 5, 64]);
    }

    #[test]
    fn caps_overlay_render_size() {
        assert_eq!(capped_render_size(600, 900, 600, 900), (600, 900));
        assert_eq!(capped_render_size(1200, 1800, 600, 900), (600, 900));
    }

    #[test]
    fn overlay_rect_uses_largest_portrait_fit() {
        let rect = fit_overlay_rect(
            TerminalGrid {
                cols: 160,
                rows: 60,
            },
            TerminalPixels {
                width: 1600,
                height: 1200,
            },
            160,
            60,
        );
        assert_eq!(rect.height, 60);
        assert_eq!(rect.width, 80);
        assert_eq!(rect.x, 40);
    }

    #[test]
    fn renders_overlay_at_current_frame_size() {
        let mut overlay = SplashOverlay::new("assets/svg/splash.svg".into());
        overlay
            .ensure_rendered(Path::new("assets/svg/splash.svg"), 60, 40, 0.0)
            .unwrap();
        assert!(overlay.last_state.is_some());
    }

    #[test]
    fn fade_opacity_does_not_force_svg_reraster() {
        let mut overlay = SplashOverlay::new("assets/svg/splash.svg".into());
        overlay.phase = OverlayPhase::SplashFadeOut {
            start: Instant::now() + std::time::Duration::from_secs(10),
        };
        let frame = overlay.next_frame();
        overlay
            .ensure_rendered(&frame.path, 60, 40, frame.progress)
            .unwrap();
        let first = overlay.last_state.clone();
        let frame = overlay.next_frame();
        overlay
            .ensure_rendered(&frame.path, 60, 40, frame.progress)
            .unwrap();
        assert_eq!(overlay.last_state, first);
    }
}
