use std::{
    io::{self, Read, Write},
    time::{Duration, Instant},
};

use crossterm::terminal::size;

mod config;
mod events;
mod overlay;
mod process;
mod render;
mod resize;
mod viewport;

pub(crate) use config::AsciiVideoConfig;

use super::session::TerminalSession;
use events::{VideoEvent, read_video_event};
use overlay::SplashOverlay;
use process::{AudioPlayback, FfmpegVideo};
use render::render_frame;
use resize::{VideoMetadata, resize_frame};
use viewport::RenderViewport;

const RESIZE_POLL: Duration = Duration::from_millis(100);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum VideoExit {
    Quit,
    Start,
    Finished,
}

pub(crate) fn run_ascii_video(config: AsciiVideoConfig) -> Result<VideoExit, String> {
    let (cols, rows) = size().map_err(|error| error.to_string())?;
    let viewport = RenderViewport::new(cols.max(1), rows.max(1), config.max_render_cells);

    let mut terminal = TerminalSession::enter(false, true)?;
    play_frames(&mut terminal, &config, viewport)
}

pub(crate) fn run_ascii_splash_demo() -> Result<(), String> {
    run_ascii_video(AsciiVideoConfig::ascii("assets/video/spash-pc.mov")).map(|_| ())
}

pub(crate) fn run_rgb_splash_demo() -> Result<VideoExit, String> {
    run_ascii_video(AsciiVideoConfig::splash("assets/video/spash-pc.mov"))
}

fn play_frames(
    terminal: &mut TerminalSession,
    config: &AsciiVideoConfig,
    mut viewport: RenderViewport,
) -> Result<VideoExit, String> {
    let fps = config.fps.max(1);
    let frame_step = Duration::from_secs_f64(1.0 / f64::from(fps));
    let metadata = VideoMetadata::probe(&config.video_path)?;
    let mut ffmpeg = FfmpegVideo::spawn_source(&config.video_path, fps)?;
    let mut source_frame = vec![0u8; metadata.frame_len()];
    let mut frame = vec![0u8; viewport.frame_len()];
    let mut overlay = config.overlay_path.clone().map(SplashOverlay::new);
    if let Some(overlay) = overlay.as_ref() {
        terminal.register_image(overlay.image());
    }
    let mut output =
        Vec::with_capacity((viewport.pixel_width * viewport.cell_rows as u32 * 24) as usize);
    let mut frame_index = 0u64;
    let mut resize_state = ResizeState::new()?;
    let mut clear_next_frame = false;
    terminal
        .stdout()
        .write_all(b"\x1b[2J")
        .map_err(|error| error.to_string())?;
    let mut audio = config
        .audio
        .then(|| AudioPlayback::spawn(&config.video_path))
        .transpose()?;
    let playback_start = Instant::now();

    loop {
        if let Some(exit) =
            apply_pending_video_event(&mut resize_state, &mut audio, overlay.as_mut())?
        {
            return Ok(exit);
        }
        if resize_state.poll_due()? {
            resize_state.poll_terminal()?;
        }
        if let Some((cols, rows)) = resize_state.ready() {
            resize_playback(
                &mut viewport,
                &mut frame,
                &mut output,
                cols,
                rows,
                config.max_render_cells,
            );
            if let Some(audio) = audio.as_mut() {
                audio.restart_if_finished(&config.video_path)?;
            }
            clear_next_frame = true;
        }

        let target_frame = elapsed_frames(playback_start, fps);
        while frame_index < target_frame {
            if !read_video_frame(&mut ffmpeg, &mut source_frame)? {
                return Ok(VideoExit::Finished);
            }
            frame_index += 1;
        }
        if !read_video_frame(&mut ffmpeg, &mut source_frame)? {
            break;
        }
        frame_index += 1;

        resize_frame(
            &source_frame,
            metadata.width,
            metadata.height,
            &mut frame,
            viewport.pixel_width,
            viewport.pixel_height,
        )?;
        render_frame(config.mode, &frame, viewport, &mut output, clear_next_frame);
        clear_next_frame = false;
        terminal
            .stdout()
            .write_all(&output)
            .map_err(|error| error.to_string())?;
        if let Some(overlay) = overlay.as_mut() {
            overlay.present_overlay(
                terminal.stdout(),
                resize_state.last_size.0,
                resize_state.last_size.1,
            )?;
            if overlay.is_finished() {
                return Ok(VideoExit::Start);
            }
        }
        terminal
            .stdout()
            .flush()
            .map_err(|error| error.to_string())?;

        sleep_until(frame_deadline(playback_start, frame_step, frame_index));
    }

    Ok(VideoExit::Finished)
}

fn read_video_frame(ffmpeg: &mut FfmpegVideo, frame: &mut [u8]) -> Result<bool, String> {
    match ffmpeg.stdout.read_exact(frame) {
        Ok(()) => Ok(true),
        Err(error) if error.kind() == io::ErrorKind::UnexpectedEof => Ok(false),
        Err(error) => Err(error.to_string()),
    }
}

fn elapsed_frames(start: Instant, fps: u32) -> u64 {
    (start.elapsed().as_secs_f64() * f64::from(fps)).floor() as u64
}

fn frame_deadline(start: Instant, frame_step: Duration, frame_index: u64) -> Instant {
    start + Duration::from_secs_f64(frame_step.as_secs_f64() * frame_index as f64)
}

fn apply_pending_video_event(
    resize_state: &mut ResizeState,
    audio: &mut Option<AudioPlayback>,
    overlay: Option<&mut SplashOverlay>,
) -> Result<Option<VideoExit>, String> {
    match read_video_event()? {
        VideoEvent::Exit => Ok(Some(VideoExit::Quit)),
        VideoEvent::Mute => {
            *audio = None;
            Ok(None)
        }
        VideoEvent::Start => {
            if let Some(overlay) = overlay {
                overlay.trigger_start();
                Ok(None)
            } else {
                Ok(Some(VideoExit::Start))
            }
        }
        VideoEvent::Resize(cols, rows) => {
            resize_state.mark(cols, rows);
            if let Some(overlay) = overlay {
                overlay.invalidate_layout();
            }
            Ok(None)
        }
        VideoEvent::None => Ok(None),
    }
}

fn resize_playback(
    viewport: &mut RenderViewport,
    frame: &mut Vec<u8>,
    output: &mut Vec<u8>,
    cols: u16,
    rows: u16,
    max_render_cells: Option<u32>,
) {
    let next = RenderViewport::new(cols.max(1), rows.max(1), max_render_cells);
    if viewport.pixel_width != next.pixel_width || viewport.pixel_height != next.pixel_height {
        frame.resize(next.frame_len(), 0);
        output.clear();
        output.reserve((next.pixel_width * next.cell_rows as u32 * 24) as usize);
    }
    *viewport = next;
}

fn sleep_until(next_frame: Instant) {
    let now = Instant::now();
    if next_frame > now {
        std::thread::sleep(next_frame - now);
    }
}

struct ResizeState {
    last_size: (u16, u16),
    last_poll: Instant,
    pending: Option<(u16, u16)>,
}

impl ResizeState {
    fn new() -> Result<Self, String> {
        let last_size = size().unwrap_or((80, 24));
        Ok(Self {
            last_size,
            last_poll: Instant::now(),
            pending: None,
        })
    }

    fn mark(&mut self, cols: u16, rows: u16) {
        self.last_size = (cols, rows);
        self.pending = Some((cols, rows));
    }

    fn poll_due(&self) -> Result<bool, String> {
        Ok(self.last_poll.elapsed() >= RESIZE_POLL)
    }

    fn poll_terminal(&mut self) -> Result<(), String> {
        if let Ok(current) = size() {
            if current != self.last_size {
                self.mark(current.0, current.1);
            }
        }
        self.last_poll = Instant::now();
        Ok(())
    }

    fn ready(&mut self) -> Option<(u16, u16)> {
        self.pending.take()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn frame_deadline_tracks_rendered_frame_index() {
        let start = Instant::now();
        let deadline = frame_deadline(start, Duration::from_millis(10), 3);
        assert_eq!(deadline.duration_since(start), Duration::from_millis(30));
    }

    #[test]
    fn resize_state_is_ready_immediately() {
        let mut state = ResizeState::new().unwrap();
        state.mark(120, 40);
        assert_eq!(state.ready(), Some((120, 40)));
    }
}
