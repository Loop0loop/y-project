use std::{
    env,
    fs::File,
    io::{self, Write},
    os::fd::AsRawFd,
    thread,
    time::Duration,
};

#[cfg(target_os = "macos")]
const TIOCGWINSZ: usize = 0x4008_7468;

#[cfg(target_os = "linux")]
const TIOCGWINSZ: usize = 0x5413;

#[cfg(not(any(target_os = "macos", target_os = "linux")))]
const TIOCGWINSZ: usize = 0;

#[repr(C)]
#[derive(Default)]
struct Winsize {
    ws_row: u16,
    ws_col: u16,
    ws_xpixel: u16,
    ws_ypixel: u16,
}

unsafe extern "C" {
    fn ioctl(fd: i32, request: usize, ...) -> i32;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct TerminalGrid {
    pub(crate) cols: u16,
    pub(crate) rows: u16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct TerminalPixels {
    pub(crate) width: u16,
    pub(crate) height: u16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum MetricSource {
    Ioctl,
    Env,
    Unknown,
}

impl MetricSource {
    fn as_str(self) -> &'static str {
        match self {
            Self::Ioctl => "ioctl",
            Self::Env => "env",
            Self::Unknown => "unknown",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct TerminalMetrics {
    pub(crate) grid: Option<TerminalGrid>,
    pub(crate) pixels: Option<TerminalPixels>,
    source: MetricSource,
}

pub(crate) fn print_probe(metrics: TerminalMetrics) {
    match metrics.grid {
        Some(grid) => println!("grid={}x{}", grid.cols, grid.rows),
        None => println!("grid=unknown"),
    }

    match metrics.pixels {
        Some(pixels) => println!("pixels={}x{}", pixels.width, pixels.height),
        None => println!("pixels=unknown"),
    }

    match (metrics.grid, metrics.pixels) {
        (Some(grid), Some(pixels)) => {
            println!(
                "cell={:.3}x{:.3}",
                f64::from(pixels.width) / f64::from(grid.cols),
                f64::from(pixels.height) / f64::from(grid.rows)
            );
        }
        _ => println!("cell=unknown"),
    }

    println!("source={}", metrics.source.as_str());
    println!("backend_candidates={}", backend_candidates());
}

pub(crate) fn probe_terminal() -> TerminalMetrics {
    if let Some(metrics) = probe_terminal_fd(io::stdout().as_raw_fd()) {
        return metrics;
    }

    if let Ok(tty) = File::open("/dev/tty")
        && let Some(metrics) = probe_terminal_fd(tty.as_raw_fd())
    {
        return metrics;
    }

    let grid = env_u16("COLUMNS")
        .zip(env_u16("LINES"))
        .map(|(cols, rows)| TerminalGrid { cols, rows });
    TerminalMetrics {
        grid,
        pixels: None,
        source: if grid.is_some() {
            MetricSource::Env
        } else {
            MetricSource::Unknown
        },
    }
}

pub(crate) fn run_watch_metrics() -> Result<(), String> {
    let mut last = None;
    for tick in 0..120 {
        let metrics = probe_terminal();
        let changed = last.is_none_or(|previous| previous != metrics);
        print!(
            "\r{} tick={tick:03} ",
            if changed { "changed" } else { "stable " }
        );
        print_metric_line(metrics);
        io::stdout().flush().map_err(|error| error.to_string())?;
        last = Some(metrics);
        thread::sleep(Duration::from_millis(250));
    }
    println!();
    Ok(())
}

fn probe_terminal_fd(fd: i32) -> Option<TerminalMetrics> {
    let mut winsize = Winsize::default();
    let ioctl_ok = TIOCGWINSZ != 0 && unsafe { ioctl(fd, TIOCGWINSZ, &mut winsize) } == 0;
    (ioctl_ok && winsize.ws_col > 0 && winsize.ws_row > 0).then_some(TerminalMetrics {
        grid: Some(TerminalGrid {
            cols: winsize.ws_col,
            rows: winsize.ws_row,
        }),
        pixels: (winsize.ws_xpixel > 0 && winsize.ws_ypixel > 0).then_some(TerminalPixels {
            width: winsize.ws_xpixel,
            height: winsize.ws_ypixel,
        }),
        source: MetricSource::Ioctl,
    })
}

fn env_u16(name: &str) -> Option<u16> {
    env::var(name).ok()?.parse().ok()
}

fn backend_candidates() -> &'static str {
    if env::var_os("KITTY_WINDOW_ID").is_some() {
        "kitty-direct,text"
    } else if env::var_os("WT_SESSION").is_some() {
        "sixel-probe-required,text"
    } else {
        "text"
    }
}

fn print_metric_line(metrics: TerminalMetrics) {
    match (metrics.grid, metrics.pixels) {
        (Some(grid), Some(pixels)) => print!(
            "grid={}x{} pixels={}x{} cell={:.3}x{:.3} source={}        ",
            grid.cols,
            grid.rows,
            pixels.width,
            pixels.height,
            f64::from(pixels.width) / f64::from(grid.cols),
            f64::from(pixels.height) / f64::from(grid.rows),
            metrics.source.as_str()
        ),
        (Some(grid), None) => print!(
            "grid={}x{} pixels=unknown cell=unknown source={}        ",
            grid.cols,
            grid.rows,
            metrics.source.as_str()
        ),
        _ => print!("grid=unknown pixels=unknown cell=unknown source=unknown        "),
    }
}
