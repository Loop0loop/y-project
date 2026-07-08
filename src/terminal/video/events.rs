use std::time::Duration;

use crossterm::event::{self, Event, KeyCode, KeyModifiers};

pub(super) enum VideoEvent {
    None,
    Exit,
    Mute,
    Start,
    Resize(u16, u16),
}

pub(super) fn read_video_event() -> Result<VideoEvent, String> {
    while event::poll(Duration::from_millis(0)).map_err(|error| error.to_string())? {
        match event::read().map_err(|error| error.to_string())? {
            Event::Key(key)
                if matches!(key.code, KeyCode::Esc | KeyCode::Char('q'))
                    || (key.code == KeyCode::Char('c')
                        && key.modifiers.contains(KeyModifiers::CONTROL)) =>
            {
                return Ok(VideoEvent::Exit);
            }
            Event::Key(key) if matches!(key.code, KeyCode::Char('m') | KeyCode::Char('M')) => {
                return Ok(VideoEvent::Mute);
            }
            Event::Key(key)
                if matches!(key.code, KeyCode::Enter) && key.modifiers == KeyModifiers::NONE =>
            {
                return Ok(VideoEvent::Start);
            }
            Event::Resize(cols, rows) => return Ok(VideoEvent::Resize(cols, rows)),
            _ => {}
        }
    }
    Ok(VideoEvent::None)
}
