use std::{
    error::Error,
    io,
    time::{Duration, Instant},
};

use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use image::{DynamicImage, Rgba, RgbaImage};
use ratatui::{
    Frame, Terminal,
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect, Size},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Gauge, List, ListItem, Paragraph, Wrap},
};
use ratatui_image::{Image, Resize, picker::Picker, protocol::Protocol};

const TICK: Duration = Duration::from_millis(66);

fn main() -> Result<(), Box<dyn Error>> {
    let mut stdout = io::stdout();
    let picker = Picker::from_query_stdio().unwrap_or_else(|_| Picker::halfblocks());

    enable_raw_mode()?;
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    let result = run(&mut terminal, picker);

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    result
}

fn run(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>, picker: Picker) -> Result<(), Box<dyn Error>> {
    let mut app = App::new(picker)?;
    let mut last_tick = Instant::now();

    loop {
        terminal.draw(|frame| draw(frame, &app))?;

        let timeout = TICK.saturating_sub(last_tick.elapsed());
        if event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                if app.on_key(key.code) {
                    return Ok(());
                }
            }
        }

        if last_tick.elapsed() >= TICK {
            app.tick();
            last_tick = Instant::now();
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Phase {
    Splash,
    Training,
    Court,
    Dating,
}

struct Stats {
    logic: u16,
    mental: u16,
    speech: u16,
    guts: u16,
    intellect: u16,
}

impl Stats {
    fn total(&self) -> u16 {
        self.logic + self.mental + self.speech + self.guts + self.intellect
    }
}

struct Training {
    name: &'static str,
    stat: &'static str,
    gain: u16,
}

struct GameState {
    phase: Phase,
    week: u16,
    energy: u16,
    affection: u16,
    selected: usize,
    court_tick: u16,
    control: i16,
    stats: Stats,
    logs: Vec<String>,
    stream_words: Vec<&'static str>,
    visible_words: usize,
}

struct App {
    state: GameState,
    trainings: Vec<Training>,
    furina: Protocol,
    prosecutor: Protocol,
    cafe: Protocol,
    protocol_name: String,
}

impl App {
    fn new(mut picker: Picker) -> Result<Self, Box<dyn Error>> {
        let font = picker.font_size();
        let portrait_size = Size::new(28, 18);
        let wide_size = Size::new(70, 22);
        let furina = picker.new_protocol(make_portrait([226, 245, 255], [36, 100, 220]), portrait_size, Resize::Fit(None))?;
        let prosecutor =
            picker.new_protocol(make_portrait([255, 232, 226], [210, 64, 76]), portrait_size, Resize::Fit(None))?;
        let cafe = picker.new_protocol(make_cafe(), wide_size, Resize::Fit(None))?;
        let protocol_name = format!("{:?} {}x{}", picker.protocol_type(), font.width, font.height);

        Ok(Self {
            state: GameState {
                phase: Phase::Splash,
                week: 1,
                energy: 3,
                affection: 10,
                selected: 0,
                court_tick: 0,
                control: 42,
                stats: Stats {
                    logic: 28,
                    mental: 35,
                    speech: 32,
                    guts: 24,
                    intellect: 30,
                },
                logs: vec!["Court engine idle. Press Enter from training when ready.".into()],
                stream_words: vec![
                    "I", "expected", "a", "more", "graceful", "victory,", "but", "your", "argument", "did", "have",
                    "a", "certain", "spark.", "Do", "not", "look", "so", "pleased.", "I", "am", "only", "being",
                    "accurate.",
                ],
                visible_words: 0,
            },
            trainings: vec![
                Training {
                    name: "Cross-exam Drill",
                    stat: "Logic Speed",
                    gain: 8,
                },
                Training {
                    name: "Public Recital",
                    stat: "Speech Power",
                    gain: 8,
                },
                Training {
                    name: "Code Review",
                    stat: "Intellect",
                    gain: 8,
                },
                Training {
                    name: "Stage Nerves",
                    stat: "Guts",
                    gain: 8,
                },
            ],
            furina,
            prosecutor,
            cafe,
            protocol_name,
        })
    }

    fn on_key(&mut self, code: KeyCode) -> bool {
        match code {
            KeyCode::Char('q') | KeyCode::Esc => return true,
            KeyCode::Tab => self.next_phase(),
            KeyCode::Up => self.state.selected = self.state.selected.saturating_sub(1),
            KeyCode::Down => self.state.selected = (self.state.selected + 1).min(self.trainings.len() - 1),
            KeyCode::Enter => self.activate(),
            _ => {}
        }
        false
    }

    fn tick(&mut self) {
        match self.state.phase {
            Phase::Splash => {}
            Phase::Court => self.tick_court(),
            Phase::Dating => {
                if self.state.visible_words < self.state.stream_words.len() {
                    self.state.visible_words += 1;
                }
            }
            Phase::Training => {}
        }
    }

    fn activate(&mut self) {
        match self.state.phase {
            Phase::Splash => self.state.phase = Phase::Training,
            Phase::Training => self.train(),
            Phase::Court => self.state.phase = Phase::Dating,
            Phase::Dating => self.state.visible_words = 0,
        }
    }

    fn next_phase(&mut self) {
        self.state.phase = match self.state.phase {
            Phase::Splash => Phase::Training,
            Phase::Training => Phase::Court,
            Phase::Court => Phase::Dating,
            Phase::Dating => Phase::Training,
        };
    }

    fn train(&mut self) {
        if self.state.energy == 0 {
            self.state.phase = Phase::Court;
            return;
        }
        let training = &self.trainings[self.state.selected];
        match training.stat {
            "Logic Speed" => self.state.stats.logic += training.gain,
            "Speech Power" => self.state.stats.speech += training.gain,
            "Intellect" => self.state.stats.intellect += training.gain,
            "Guts" => self.state.stats.guts += training.gain,
            _ => {}
        }
        self.state.energy -= 1;
        self.state.week += 1;
        if self.state.energy == 0 {
            self.state.phase = Phase::Court;
        }
    }

    fn tick_court(&mut self) {
        self.state.court_tick = self.state.court_tick.saturating_add(1);
        if self.state.court_tick % 9 != 0 {
            return;
        }

        let pressure = 36 + (self.state.court_tick / 9) * 5;
        let swing = court_swing(self.state.stats.total(), pressure);
        self.state.control = (self.state.control + swing).clamp(0, 100);
        let line = if swing >= 0 {
            format!("Furina counters cleanly. momentum +{swing}")
        } else {
            format!("Prosecutor lands pressure. momentum {swing}")
        };
        self.state.logs.push(line);
        if self.state.logs.len() > 8 {
            self.state.logs.remove(0);
        }
        if self.state.court_tick > 90 {
            self.state.phase = Phase::Dating;
        }
    }
}

fn court_swing(total_stats: u16, pressure: u16) -> i16 {
    ((total_stats as i16 - pressure as i16) / 9).clamp(-12, 12)
}

fn draw(frame: &mut Frame, app: &App) {
    let area = frame.area();
    match app.state.phase {
        Phase::Splash => draw_splash(frame, area, app),
        Phase::Training => draw_training(frame, area, app),
        Phase::Court => draw_court(frame, area, app),
        Phase::Dating => draw_dating(frame, area, app),
    }
}

fn draw_splash(frame: &mut Frame, area: Rect, app: &App) {
    frame.render_widget(Block::default().style(Style::default().bg(Color::Rgb(8, 17, 36))), area);
    constellation(frame, area);
    let title = Paragraph::new(vec![
        Line::from(Span::styled(
            "FURINA ADVOCATE SIMULATOR",
            Style::default()
                .fg(Color::Rgb(236, 248, 255))
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(Span::styled(
            format!("graphics: {}", app.protocol_name),
            Style::default().fg(Color::Rgb(143, 211, 255)),
        )),
        Line::from(""),
        Line::from("Enter start  |  Tab skip phase  |  q quit"),
    ])
    .centered();
    frame.render_widget(title, centered(area, 52, 9));
}

fn draw_training(frame: &mut Frame, area: Rect, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(24), Constraint::Min(40), Constraint::Length(34)])
        .split(area);

    let menu = List::new([
        ListItem::new("Training"),
        ListItem::new("Stats"),
        ListItem::new("Schedule"),
        ListItem::new("Court Prep"),
    ])
    .block(Block::new().borders(Borders::RIGHT).title("MENU"))
    .style(Style::default().fg(Color::Rgb(179, 210, 235)));
    frame.render_widget(menu, chunks[0]);

    let stats = &app.state.stats;
    let stat_rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3); 5])
        .split(chunks[1]);
    stat_bar(frame, stat_rows[0], "Logic Speed", stats.logic);
    stat_bar(frame, stat_rows[1], "Mental Stamina", stats.mental);
    stat_bar(frame, stat_rows[2], "Speech Power", stats.speech);
    stat_bar(frame, stat_rows[3], "Guts", stats.guts);
    stat_bar(frame, stat_rows[4], "Intellect", stats.intellect);

    let items: Vec<ListItem> = app
        .trainings
        .iter()
        .enumerate()
        .map(|(idx, item)| {
            let marker = if idx == app.state.selected { "> " } else { "  " };
            ListItem::new(format!("{marker}{}  +{} {}", item.name, item.gain, item.stat))
        })
        .collect();
    let schedule = List::new(items)
        .block(
            Block::bordered()
                .title(format!("Week {} / Energy {}", app.state.week, app.state.energy))
                .style(Style::default().fg(Color::Rgb(92, 148, 190))),
        )
        .style(Style::default().fg(Color::Rgb(230, 239, 246)));
    frame.render_widget(schedule, chunks[2]);
}

fn draw_court(frame: &mut Frame, area: Rect, app: &App) {
    frame.render_widget(Block::default().style(Style::default().bg(Color::Rgb(13, 20, 31))), area);
    slash(frame, area);
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(35), Constraint::Percentage(30), Constraint::Percentage(35)])
        .split(area);
    frame.render_widget(Image::new(&app.prosecutor), inset(chunks[0], 2, 2));
    frame.render_widget(Image::new(&app.furina), inset(chunks[2], 2, 2));

    let center = chunks[1];
    let objection = if app.state.control > 62 { "OBJECTION!" } else { "ARGUE!" };
    let text = Paragraph::new(vec![
        Line::from(Span::styled(
            objection,
            Style::default()
                .fg(Color::Rgb(255, 246, 114))
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(format!("control {}%", app.state.control)),
    ])
    .centered()
    .block(Block::bordered().style(Style::default().fg(Color::Rgb(255, 246, 114))));
    frame.render_widget(text, centered(center, 24, 7));

    let log = Paragraph::new(app.state.logs.join("\n"))
        .wrap(Wrap { trim: true })
        .block(Block::bordered().title("LIVE COURT LOG"));
    frame.render_widget(log, Rect::new(area.x + 3, area.y + area.height.saturating_sub(10), area.width - 6, 9));
}

fn draw_dating(frame: &mut Frame, area: Rect, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(70), Constraint::Percentage(30)])
        .split(area);
    frame.render_widget(Image::new(&app.cafe), chunks[0]);
    frame.render_widget(Image::new(&app.furina), Rect::new(chunks[0].x + chunks[0].width.saturating_sub(34), chunks[0].y + 1, 30, 18));

    let words = app.state.stream_words[..app.state.visible_words].join(" ");
    let dialog = Paragraph::new(vec![
        Line::from(Span::styled(
            format!("Furina  Affection {}", app.state.affection),
            Style::default()
                .fg(Color::Rgb(97, 169, 238))
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(words),
        Line::from(""),
        Line::from("> "),
    ])
    .wrap(Wrap { trim: true })
    .block(Block::bordered().title("<>"));
    frame.render_widget(dialog, inset(chunks[1], 2, 1));
}

fn stat_bar(frame: &mut Frame, area: Rect, label: &'static str, value: u16) {
    let gauge = Gauge::default()
        .block(Block::new().title(label))
        .gauge_style(Style::default().fg(Color::Rgb(81, 184, 221)).bg(Color::Rgb(27, 43, 64)))
        .ratio((value.min(100) as f64) / 100.0)
        .label(format!("{value:03}"));
    frame.render_widget(gauge, area);
}

fn constellation(frame: &mut Frame, area: Rect) {
    for row in (2..area.height.saturating_sub(2)).step_by(4) {
        for col in (4..area.width.saturating_sub(4)).step_by(11) {
            let dot = Rect::new(area.x + col, area.y + row, 1, 1);
            frame.render_widget(Paragraph::new("*").style(Style::default().fg(Color::Rgb(62, 122, 178))), dot);
        }
    }
}

fn slash(frame: &mut Frame, area: Rect) {
    for y in 0..area.height {
        let x = area.x + (u32::from(y) * u32::from(area.width) / u32::from(area.height.max(1))) as u16;
        frame.render_widget(
            Paragraph::new("/").style(
                Style::default()
                    .fg(Color::Rgb(255, 246, 114))
                    .add_modifier(Modifier::BOLD),
            ),
            Rect::new(x, area.y + y, 1, 1),
        );
    }
}

fn centered(area: Rect, width: u16, height: u16) -> Rect {
    Rect::new(
        area.x + area.width.saturating_sub(width) / 2,
        area.y + area.height.saturating_sub(height) / 2,
        width.min(area.width),
        height.min(area.height),
    )
}

fn inset(area: Rect, x: u16, y: u16) -> Rect {
    Rect::new(
        area.x + x,
        area.y + y,
        area.width.saturating_sub(x * 2),
        area.height.saturating_sub(y * 2),
    )
}

fn make_portrait(base: [u8; 3], accent: [u8; 3]) -> DynamicImage {
    let mut img = RgbaImage::new(480, 360);
    for y in 0..360 {
        for x in 0..480 {
            let t = y as f32 / 360.0;
            let glow = (((x as i32 - 240).abs() + (y as i32 - 170).abs()) as f32 / 520.0).min(1.0);
            img.put_pixel(
                x,
                y,
                Rgba([
                    lerp(base[0], accent[0], t * 0.45 + glow * 0.25),
                    lerp(base[1], accent[1], t * 0.45 + glow * 0.25),
                    lerp(base[2], accent[2], t * 0.45 + glow * 0.25),
                    255,
                ]),
            );
        }
    }
    for y in 80..315 {
        for x in 145..335 {
            let dx = (x as i32 - 240).abs();
            let dy = (y as i32 - 190).abs();
            if dx * 2 + dy < 185 {
                img.put_pixel(x, y, Rgba([245, 250, 255, 235]));
            }
        }
    }
    for y in 40..145 {
        for x in 118..362 {
            let dx = (x as i32 - 240).abs();
            let dy = (y as i32 - 118).abs();
            if dx + dy * 2 < 170 {
                img.put_pixel(x, y, Rgba([accent[0], accent[1], accent[2], 255]));
            }
        }
    }
    DynamicImage::ImageRgba8(img)
}

fn make_cafe() -> DynamicImage {
    let mut img = RgbaImage::new(960, 420);
    for y in 0..420 {
        for x in 0..960 {
            let sky = y < 210;
            let base = if sky { [204, 235, 251] } else { [222, 225, 218] };
            let shade = ((x + y) % 90) as u8;
            img.put_pixel(x, y, Rgba([base[0] - shade / 8, base[1] - shade / 10, base[2] - shade / 12, 255]));
        }
    }
    for y in 220..390 {
        for x in 80..880 {
            if (x / 80 + y / 40) % 2 == 0 {
                img.put_pixel(x, y, Rgba([190, 207, 219, 255]));
            }
        }
    }
    for y in 70..230 {
        for x in 95..300 {
            if x % 7 < 4 {
                img.put_pixel(x, y, Rgba([243, 247, 249, 255]));
            }
        }
        for x in 660..865 {
            if x % 7 < 4 {
                img.put_pixel(x, y, Rgba([243, 247, 249, 255]));
            }
        }
    }
    DynamicImage::ImageRgba8(img)
}

fn lerp(a: u8, b: u8, t: f32) -> u8 {
    (a as f32 + (b as f32 - a as f32) * t.clamp(0.0, 1.0)) as u8
}

#[cfg(test)]
mod tests {
    use super::court_swing;

    #[test]
    fn stronger_stats_shift_court_control_up() {
        assert!(court_swing(180, 60) > 0);
        assert!(court_swing(70, 140) < 0);
        assert_eq!(court_swing(400, 0), 12);
    }
}
