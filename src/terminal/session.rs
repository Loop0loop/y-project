use std::{
    io::{self, Write},
    panic,
    sync::Once,
    time::{Duration, Instant},
};

use crossterm::{
    cursor::{Hide, Show},
    event::{self, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{
        Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode,
        enable_raw_mode,
    },
};

use super::kitty::{KittyImage, delete_image};

static PANIC_RESTORE_HOOK: Once = Once::new();

pub(crate) struct TerminalSession {
    stdout: io::Stdout,
    raw: bool,
    alternate: bool,
    cursor_hidden: bool,
    wrap_disabled: bool,
    images: Vec<KittyImage>,
}

impl TerminalSession {
    pub(crate) fn enter(clear: bool, disable_wrap: bool) -> Result<Self, String> {
        install_panic_restore_hook();

        let mut session = Self {
            stdout: io::stdout(),
            raw: false,
            alternate: false,
            cursor_hidden: false,
            wrap_disabled: false,
            images: Vec::new(),
        };

        enable_raw_mode().map_err(|error| error.to_string())?;
        session.raw = true;
        execute!(session.stdout, EnterAlternateScreen).map_err(|error| error.to_string())?;
        session.alternate = true;
        execute!(session.stdout, Hide).map_err(|error| error.to_string())?;
        session.cursor_hidden = true;
        if clear {
            execute!(session.stdout, Clear(ClearType::All)).map_err(|error| error.to_string())?;
        }
        if disable_wrap {
            session
                .stdout
                .write_all(b"\x1b[?7l")
                .map_err(|error| error.to_string())?;
            session.wrap_disabled = true;
        }

        Ok(session)
    }

    pub(crate) fn stdout(&mut self) -> &mut io::Stdout {
        &mut self.stdout
    }

    pub(crate) fn register_image(&mut self, image: KittyImage) {
        if !self.images.contains(&image) {
            self.images.push(image);
        }
    }

    fn restore(&mut self) {
        for image in self.images.drain(..).rev() {
            let _ = delete_image(&mut self.stdout, &image);
        }
        if self.wrap_disabled {
            let _ = self.stdout.write_all(b"\x1b[0m\x1b[?7h");
            self.wrap_disabled = false;
        }
        if self.cursor_hidden {
            let _ = execute!(self.stdout, Show);
            self.cursor_hidden = false;
        }
        if self.alternate {
            let _ = execute!(self.stdout, LeaveAlternateScreen);
            self.alternate = false;
        }
        if self.raw {
            let _ = disable_raw_mode();
            self.raw = false;
        }
        let _ = self.stdout.flush();
    }
}

impl Drop for TerminalSession {
    fn drop(&mut self) {
        self.restore();
    }
}

pub(crate) fn wait_or_interrupt(duration: Duration) -> Result<bool, String> {
    let deadline = Instant::now() + duration;
    loop {
        let now = Instant::now();
        if now >= deadline {
            return Ok(false);
        }
        let timeout = (deadline - now).min(Duration::from_millis(50));
        if !event::poll(timeout).map_err(|error| error.to_string())? {
            continue;
        }
        if matches!(
            event::read().map_err(|error| error.to_string())?,
            Event::Key(key)
                if key.code == KeyCode::Esc
                    || key.code == KeyCode::Char('q')
                    || (key.code == KeyCode::Char('c')
                        && key.modifiers.contains(KeyModifiers::CONTROL))
        ) {
            return Ok(true);
        }
    }
}

fn install_panic_restore_hook() {
    PANIC_RESTORE_HOOK.call_once(|| {
        let previous = panic::take_hook();
        panic::set_hook(Box::new(move |info| {
            let mut stdout = io::stdout();
            let _ = stdout.write_all(b"\x1b[0m\x1b[?7h\x1b[?25h\x1b[?1049l");
            let _ = stdout.flush();
            let _ = disable_raw_mode();
            previous(info);
        }));
    });
}
