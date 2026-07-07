use std::path::{Path, PathBuf};

pub(super) const MAX_RENDER_CELLS: u32 = 12_000;
const SPLASH_FPS: u32 = 24;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum VideoRenderMode {
    Ascii,
    Rgb,
}

#[derive(Debug, Clone)]
pub(crate) struct AsciiVideoConfig {
    pub(crate) video_path: PathBuf,
    pub(crate) fps: u32,
    pub(crate) mode: VideoRenderMode,
    pub(crate) max_render_cells: Option<u32>,
    pub(crate) audio: bool,
    pub(crate) overlay_path: Option<PathBuf>,
}

impl AsciiVideoConfig {
    pub(crate) fn new(video_path: impl AsRef<Path>) -> Self {
        Self {
            video_path: video_path.as_ref().to_path_buf(),
            fps: 30,
            mode: VideoRenderMode::Rgb,
            max_render_cells: None,
            audio: true,
            overlay_path: None,
        }
    }

    pub(crate) fn ascii(video_path: impl AsRef<Path>) -> Self {
        Self {
            mode: VideoRenderMode::Ascii,
            max_render_cells: Some(MAX_RENDER_CELLS),
            ..Self::new(video_path)
        }
    }

    pub(crate) fn splash(video_path: impl AsRef<Path>) -> Self {
        Self {
            fps: SPLASH_FPS,
            overlay_path: Some(PathBuf::from("assets/svg/splash_overlay.svg")),
            ..Self::new(video_path)
        }
    }
}
