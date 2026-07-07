use std::{
    io::{self, Read, Write},
    time::{Duration, Instant},
};

use crossterm::{
    cursor::{Hide, Show},
    execute,
    terminal::{
        disable_raw_mode, enable_raw_mode, size, EnterAlternateScreen, LeaveAlternateScreen,
    },
};

mod config;
mod events;
mod overlay;
mod process;
mod render;
mod resize;
mod viewport;

pub(crate) use config::AsciiVideoConfig;

use events::{read_video_event, VideoEvent};
use overlay::SplashOverlay;
use process::{AudioPlayback, FfmpegVideo};
use render::render_frame;
use resize::{resize_frame, VideoMetadata};
use viewport::RenderViewport;

const RESIZE_POLL: Duration = Duration::from_millis(100);
const RESIZE_DEBOUNCE: Duration = Duration::from_millis(150);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum VideoExit {
    Quit,
    Start,
    Finished,
}

pub(crate) fn run_ascii_video(config: AsciiVideoConfig) -> Result<VideoExit, String> {
    let (cols, rows) = size().map_err(|error| error.to_string())?;
    let viewport = RenderViewport::new(cols.max(1), rows.max(1), config.max_render_cells);

    let mut stdout = io::stdout();
    enable_raw_mode().map_err(|error| error.to_string())?;
    let mut guard = TerminalGuard;
    execute!(stdout, EnterAlternateScreen, Hide).map_err(|error| error.to_string())?;

    let result = play_frames(&mut stdout, &config, viewport);
    guard.restore(&mut stdout);
    result
}

pub(crate) fn run_ascii_splash_demo() -> Result<(), String> {
    run_ascii_video(AsciiVideoConfig::ascii("assets/video/spash-pc.mov")).map(|_| ())
}

pub(crate) fn run_rgb_splash_demo() -> Result<VideoExit, String> {
    run_ascii_video(AsciiVideoConfig::splash("assets/video/spash-pc.mov"))
}

fn play_frames(
    stdout: &mut io::Stdout,
    config: &AsciiVideoConfig,
    mut viewport: RenderViewport,
) -> Result<VideoExit, String> {
    let fps = config.fps.max(1);
    let frame_step = Duration::from_secs_f64(1.0 / f64::from(fps));
    let metadata = VideoMetadata::probe(&config.video_path)?;
    let mut ffmpeg = FfmpegVideo::spawn_source(&config.video_path, fps)?;
    let mut source_frame = vec![0u8; metadata.frame_len()];
    let mut frame = vec![0u8; viewport.frame_len()];
    let mut overlay = SplashOverlay::new(config.overlay_path.clone());
    let mut output =
        Vec::with_capacity((viewport.pixel_width * viewport.cell_rows as u32 * 24) as usize);
    let mut frame_index = 0u64;
    let mut resize_state = ResizeState::new()?;
    let mut clear_next_frame = false;
    stdout
        .write_all(b"\x1b[2J\x1b[?7l")
        .map_err(|error| error.to_string())?;
    let mut audio = config
        .audio
        .then(|| AudioPlayback::spawn(&config.video_path))
        .transpose()?;
    let playback_start = Instant::now();

    loop {
        if let Some(exit) = handle_input(&mut resize_state, &mut audio)? {
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
            clear_next_frame = true;
        }

        let target_frame = elapsed_frames(playback_start, fps);
        while frame_index < target_frame {
            if !read_video_frame(&mut ffmpeg, &mut source_frame)? {
                return finish_video(stdout);
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
        overlay.blend_into(&mut frame, viewport.pixel_width, viewport.pixel_height)?;
        render_frame(config.mode, &frame, viewport, &mut output, clear_next_frame);
        clear_next_frame = false;
        stdout
            .write_all(&output)
            .map_err(|error| error.to_string())?;
        stdout.flush().map_err(|error| error.to_string())?;

        sleep_until(frame_deadline(playback_start, frame_step, frame_index));
    }

    finish_video(stdout)
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

fn finish_video(stdout: &mut io::Stdout) -> Result<VideoExit, String> {
    stdout
        .write_all(b"\x1b[0m\x1b[?7h")
        .map_err(|error| error.to_string())?;
    Ok(VideoExit::Finished)
}

fn handle_input(
    resize_state: &mut ResizeState,
    audio: &mut Option<AudioPlayback>,
) -> Result<Option<VideoExit>, String> {
    match read_video_event()? {
        VideoEvent::Exit => Ok(Some(VideoExit::Quit)),
        VideoEvent::Mute => {
            *audio = None;
            Ok(None)
        }
        VideoEvent::Start => Ok(Some(VideoExit::Start)),
        VideoEvent::Resize(cols, rows) => {
            resize_state.mark(cols, rows);
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
    pending: Option<(u16, u16, Instant)>,
}

impl ResizeState {
    fn new() -> Result<Self, String> {
        Ok(Self {
            last_size: size().map_err(|error| error.to_string())?,
            last_poll: Instant::now(),
            pending: None,
        })
    }

    fn mark(&mut self, cols: u16, rows: u16) {
        self.last_size = (cols, rows);
        self.pending = Some((cols, rows, Instant::now()));
    }

    fn poll_due(&self) -> Result<bool, String> {
        Ok(self.last_poll.elapsed() >= RESIZE_POLL)
    }

    fn poll_terminal(&mut self) -> Result<(), String> {
        let current = size().map_err(|error| error.to_string())?;
        if current != self.last_size {
            self.mark(current.0, current.1);
        }
        self.last_poll = Instant::now();
        Ok(())
    }

    fn ready(&mut self) -> Option<(u16, u16)> {
        let (cols, rows, changed_at) = self.pending?;
        if changed_at.elapsed() < RESIZE_DEBOUNCE {
            return None;
        }
        self.pending = None;
        Some((cols, rows))
    }
}

struct TerminalGuard;

impl TerminalGuard {
    fn restore(&mut self, stdout: &mut io::Stdout) {
        let _ = stdout.write_all(b"\x1b[0m\x1b[?7h");
        let _ = execute!(stdout, Show, LeaveAlternateScreen);
        let _ = disable_raw_mode();
    }
}

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        let _ = disable_raw_mode();
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
}
