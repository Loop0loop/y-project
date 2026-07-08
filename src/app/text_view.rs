use std::io::{self, Write};

use crossterm::{
    cursor::MoveTo,
    execute,
    terminal::{Clear, ClearType},
};

use crate::domain::{AdvocateStats, TRAINING_ACTIONS};

use super::{FAKE_RESPONSE, Screen, SpaApp};

pub(super) fn draw_text(stdout: &mut io::Stdout, app: &SpaApp) -> Result<(), String> {
    execute!(stdout, MoveTo(0, 0), Clear(ClearType::All)).map_err(|error| error.to_string())?;
    writeln!(stdout, "Project-Y MVP Loop  |  q/Esc quit").map_err(|error| error.to_string())?;
    writeln!(stdout, "phase={:?}", app.phase()).map_err(|error| error.to_string())?;
    writeln!(stdout).map_err(|error| error.to_string())?;

    match app.screen() {
        Screen::Splash => draw_splash(stdout, app),
        Screen::Loading => draw_loading(stdout, app),
        Screen::Training => draw_training(stdout, app),
        Screen::CourtReplay => draw_court(stdout, app),
        Screen::Dating => draw_dating(stdout, app),
        Screen::Result => draw_result(stdout, app),
    }?;

    stdout.flush().map_err(|error| error.to_string())
}

fn draw_splash(stdout: &mut io::Stdout, app: &SpaApp) -> Result<(), String> {
    writeln!(
        stdout,
        "[Splash Screen] Loading... {}%",
        app.splash_progress()
    )
    .map_err(|error| error.to_string())?;
    writeln!(stdout, "Press Enter to skip").map_err(|error| error.to_string())
}

fn draw_loading(stdout: &mut io::Stdout, app: &SpaApp) -> Result<(), String> {
    let (tip_header, tip_body) = app.loading_tip();
    writeln!(stdout, "[Loading Screen] {} - {}", tip_header, tip_body)
        .map_err(|error| error.to_string())?;
    writeln!(stdout, "Progress: {}%", app.loading_progress()).map_err(|error| error.to_string())
}

fn draw_training(stdout: &mut io::Stdout, app: &SpaApp) -> Result<(), String> {
    writeln!(stdout, "[Training] Up/Down select, Enter confirm")
        .map_err(|error| error.to_string())?;
    for (index, action) in TRAINING_ACTIONS.iter().enumerate() {
        let marker = if index == app.focused_action() {
            ">"
        } else {
            " "
        };
        writeln!(stdout, "{marker} {}", action.label).map_err(|error| error.to_string())?;
    }
    writeln!(stdout).map_err(|error| error.to_string())?;
    write_stats(stdout, app.stats())
}

fn draw_court(stdout: &mut io::Stdout, app: &SpaApp) -> Result<(), String> {
    writeln!(stdout, "[Court Replay] Enter skip").map_err(|error| error.to_string())?;
    for line in app.court_log().iter().take(app.shown_court_logs()) {
        writeln!(stdout, "- {line}").map_err(|error| error.to_string())?;
    }
    if app.shown_court_logs() == app.court_log().len() {
        writeln!(stdout, "result={:?}", app.court_result()).map_err(|error| error.to_string())?;
    }
    Ok(())
}

fn draw_dating(stdout: &mut io::Stdout, app: &SpaApp) -> Result<(), String> {
    writeln!(stdout, "[Dating] type message, Enter finish").map_err(|error| error.to_string())?;
    let response: String = FAKE_RESPONSE
        .chars()
        .take(app.visible_response_chars())
        .collect();
    for line in wrap_text(&response, 54) {
        writeln!(stdout, "Furina: {line}").map_err(|error| error.to_string())?;
    }
    writeln!(stdout).map_err(|error| error.to_string())?;
    writeln!(stdout, "> {}", app.input()).map_err(|error| error.to_string())
}

fn draw_result(stdout: &mut io::Stdout, app: &SpaApp) -> Result<(), String> {
    writeln!(stdout, "[Result] Enter exit").map_err(|error| error.to_string())?;
    writeln!(stdout, "court={:?}", app.court_result()).map_err(|error| error.to_string())?;
    writeln!(stdout, "transcript_len={}", app.transcript_len()).map_err(|error| error.to_string())
}

fn write_stats(stdout: &mut io::Stdout, stats: AdvocateStats) -> Result<(), String> {
    for (label, value) in [
        ("Logic Speed", stats.logic_speed),
        ("Mental Stamina", stats.mental_stamina),
        ("Speech Power", stats.speech_power),
        ("Guts", stats.guts),
        ("Intellect", stats.intellect),
    ] {
        writeln!(stdout, "{label:15} {}", bar(value)).map_err(|error| error.to_string())?;
    }
    Ok(())
}

fn bar(value: u16) -> String {
    let filled = (usize::from(value.min(100)) * 20) / 100;
    format!(
        "[{}{}] {value:03}",
        "#".repeat(filled),
        ".".repeat(20 - filled)
    )
}

fn wrap_text(text: &str, width: usize) -> Vec<String> {
    let mut lines = Vec::new();
    let mut current = String::new();
    for ch in text.chars() {
        if current.chars().count() >= width {
            lines.push(std::mem::take(&mut current));
        }
        current.push(ch);
    }
    if !current.is_empty() {
        lines.push(current);
    }
    lines
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wraps_korean_and_english_without_losing_chars() {
        let text = "abc가나다def";
        let lines = wrap_text(text, 4);
        assert_eq!(lines.concat(), text);
        assert!(lines.iter().all(|line| line.chars().count() <= 4));
    }
}
