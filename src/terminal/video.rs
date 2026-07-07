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
    let _audio = config
        .audio
        .then(|| AudioPlayback::spawn(&config.video_path))
        .transpose()?;
    let mut source_frame = vec![0u8; metadata.frame_len()];
    let mut frame = vec![0u8; viewport.frame_len()];
    let mut overlay = SplashOverlay::new(config.overlay_path.clone());
    let mut output =
        Vec::with_capacity((viewport.pixel_width * viewport.cell_rows as u32 * 24) as usize);
    let mut next_frame = Instant::now();
    let mut resize_state = ResizeState::new()?;
    stdout
        .write_all(b"\x1b[2J\x1b[?7l")
        .map_err(|error| error.to_string())?;

    loop {
        if let Some(exit) = handle_input(&mut resize_state)? {
            return Ok(exit);
        }
        if resize_state.poll_due()? {
            resize_state.poll_terminal()?;
        }
        if let Some((cols, rows)) = resize_state.ready() {
            resize_playback(
                stdout,
                &mut viewport,
                &mut frame,
                &mut output,
                cols,
                rows,
                config.max_render_cells,
            )?;
            next_frame = Instant::now();
        }

        match ffmpeg.stdout.read_exact(&mut source_frame) {
            Ok(()) => {}
            Err(error) if error.kind() == io::ErrorKind::UnexpectedEof => break,
            Err(error) => return Err(error.to_string()),
        }

        resize_frame(
            &source_frame,
            metadata.width,
            metadata.height,
            &mut frame,
            viewport.pixel_width,
            viewport.pixel_height,
        )?;
        overlay.blend_into(&mut frame, viewport.pixel_width, viewport.pixel_height)?;
        render_frame(config.mode, &frame, viewport, &mut output);
        stdout
            .write_all(&output)
            .map_err(|error| error.to_string())?;
        stdout.flush().map_err(|error| error.to_string())?;

        next_frame += frame_step;
        sleep_until(next_frame, &mut next_frame);
    }

    stdout
        .write_all(b"\x1b[0m\x1b[?7h")
        .map_err(|error| error.to_string())?;
    Ok(VideoExit::Finished)
}

fn handle_input(resize_state: &mut ResizeState) -> Result<Option<VideoExit>, String> {
    match read_video_event()? {
        VideoEvent::Exit => Ok(Some(VideoExit::Quit)),
        VideoEvent::Start => Ok(Some(VideoExit::Start)),
        VideoEvent::Resize(cols, rows) => {
            resize_state.mark(cols, rows);
            Ok(None)
        }
        VideoEvent::None => Ok(None),
    }
}

fn resize_playback(
    stdout: &mut io::Stdout,
    viewport: &mut RenderViewport,
    frame: &mut Vec<u8>,
    output: &mut Vec<u8>,
    cols: u16,
    rows: u16,
    max_render_cells: Option<u32>,
) -> Result<(), String> {
    let next = RenderViewport::new(cols.max(1), rows.max(1), max_render_cells);
    if viewport.pixel_width != next.pixel_width || viewport.pixel_height != next.pixel_height {
        frame.resize(next.frame_len(), 0);
        output.clear();
        output.reserve((next.pixel_width * next.cell_rows as u32 * 24) as usize);
    }
    *viewport = next;
    stdout
        .write_all(b"\x1b[0m\x1b[2J")
        .map_err(|error| error.to_string())?;
    stdout.flush().map_err(|error| error.to_string())
}

fn sleep_until(next_frame: Instant, stored_next_frame: &mut Instant) {
    let now = Instant::now();
    if next_frame > now {
        std::thread::sleep(next_frame - now);
    } else {
        *stored_next_frame = now;
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
