use std::path::{Path, PathBuf};
use std::time::Instant;

use crate::shared::{LOADING_PROGRESS_WIDTH, ease_out, escape_xml, loading_tip};
use crate::terminal::kitty::{KittyImage, present_rgba_with_z};

mod layout;
use layout::{OverlayLayout, compute_overlay_layout};

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

pub(super) struct SplashOverlay {
    rgba: Vec<u8>,
    scratch: Vec<u8>,
    image: KittyImage,
    pub(super) phase: OverlayPhase,
    last_state: Option<RenderState>,
    last_layout: Option<OverlayLayout>,
    tip_header: String,
    tip_body: String,
    splash_path: PathBuf,
}

impl SplashOverlay {
    pub(super) fn new(path: PathBuf) -> Self {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::SystemTime::UNIX_EPOCH)
            .map(|d| d.subsec_nanos())
            .unwrap_or(0);
        let tip = loading_tip(nanos);
        Self {
            rgba: Vec::new(),
            scratch: Vec::new(),
            image: KittyImage {
                image_id: 525_310,
                placement_id: 31,
            },
            phase: OverlayPhase::SplashActive,
            last_state: None,
            last_layout: None,
            tip_header: tip.0.to_string(),
            tip_body: tip.1.to_string(),
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

    pub(super) fn invalidate_layout(&mut self) {
        self.last_layout = None;
        self.last_state = None;
    }

    pub(super) fn present_overlay(
        &mut self,
        stdout: &mut impl std::io::Write,
        terminal_cols: u16,
        terminal_rows: u16,
    ) -> Result<(), String> {
        let (opacity, progress, path) = self.next_overlay_frame();
        let layout = self.cached_layout(terminal_cols, terminal_rows)?;
        let cell_rect = layout.cell_rect;
        let render_width = layout.render_width;
        let render_height = layout.render_height;
        self.render_overlay_if_stale(&path, render_width, render_height, progress)?;
        let image = self.image;
        let rgba = if opacity >= 0.995 {
            self.rgba.as_slice()
        } else {
            self.frame_with_opacity(opacity)
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
        Ok(())
    }

    fn next_overlay_frame(&mut self) -> (f32, f32, PathBuf) {
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

            return (opacity, progress, path);
        }
    }

    fn render_overlay_if_stale(
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
                &format!(
                    "{:.2}",
                    LOADING_PROGRESS_WIDTH * progress.clamp(0.0, 100.0) / 100.0
                ),
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

    fn frame_with_opacity(&mut self, opacity: f32) -> &[u8] {
        apply_opacity(&self.rgba, opacity, &mut self.scratch);
        &self.scratch
    }

    fn cached_layout(
        &mut self,
        terminal_cols: u16,
        terminal_rows: u16,
    ) -> Result<OverlayLayout, String> {
        if let Some(layout) = self.last_layout
            && layout.terminal_cols == terminal_cols
            && layout.terminal_rows == terminal_rows
        {
            return Ok(layout);
        }
        let layout = compute_overlay_layout(terminal_cols, terminal_rows)?;
        self.last_layout = Some(layout);
        Ok(layout)
    }
}

fn apply_opacity(rgba: &[u8], opacity: f32, out: &mut Vec<u8>) {
    let opacity = opacity.clamp(0.0, 1.0);
    out.clear();
    out.extend_from_slice(rgba);
    for pixel in out.chunks_exact_mut(4) {
        pixel[0] = scale_channel_by_opacity(pixel[0], opacity);
        pixel[1] = scale_channel_by_opacity(pixel[1], opacity);
        pixel[2] = scale_channel_by_opacity(pixel[2], opacity);
        pixel[3] = scale_channel_by_opacity(pixel[3], opacity);
    }
}

fn scale_channel_by_opacity(value: u8, opacity: f32) -> u8 {
    (f32::from(value) * opacity).round() as u8
}

#[cfg(test)]
#[path = "overlay_tests.rs"]
mod tests;
