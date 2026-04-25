mod parser;
mod player;

use anyhow::{Context, Result};
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use parser::{parse_m3u8, parse_m3u8_str, Station};
use player::{PlaybackStatus, Player};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Style, Stylize},
    widgets::{
        Block, Borders, List, ListItem, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState,
    },
    Frame, Terminal,
};
use std::io;
use std::path::PathBuf;

const EMBEDDED_PLAYLIST: &str = include_str!("../playlist/malaysia-radio.m3u8");

struct App {
    stations: Vec<Station>,
    selected: usize,
    scroll_offset: usize,
    playing_station: Option<String>,
    status: PlaybackStatus,
    player: Player,
    list_height: usize,
}

impl App {
    fn new() -> Result<Self> {
        Ok(Self {
            stations: load_stations()?,
            selected: 0,
            scroll_offset: 0,
            playing_station: None,
            status: PlaybackStatus::Stopped,
            player: Player::new(),
            list_height: 20,
        })
    }

    fn update_status(&mut self) {
        self.status = self.player.get_status();
    }

    fn play_station(&mut self) {
        if self.stations.is_empty() {
            return;
        }

        let station = &self.stations[self.selected];
        self.playing_station = Some(station.name.clone());
        self.status = PlaybackStatus::Loading;

        if let Err(e) = self.player.play(&station.url) {
            self.status = PlaybackStatus::Error;
            eprintln!("Error: {}", e);
        }
    }

    fn stop(&mut self) {
        self.player.stop();
        self.playing_station = None;
        self.status = PlaybackStatus::Stopped;
    }

    fn toggle_playback(&mut self) {
        if self.player.is_playing() {
            self.player.pause();
            self.status = PlaybackStatus::Paused;
        } else if self.status == PlaybackStatus::Paused {
            self.player.resume();
            self.status = PlaybackStatus::Playing;
        }
    }

    fn move_up(&mut self) {
        if !self.stations.is_empty() {
            self.selected = self.selected.saturating_sub(1);
            if self.selected < self.scroll_offset {
                self.scroll_offset = self.selected;
            }
        }
    }

    fn move_down(&mut self) {
        if !self.stations.is_empty() {
            self.selected = (self.selected + 1).min(self.stations.len() - 1);
            if self.selected >= self.scroll_offset + self.list_height {
                self.scroll_offset = self.selected.saturating_sub(self.list_height) + 1;
            }
        }
    }

    fn move_page_up(&mut self) {
        if !self.stations.is_empty() {
            let visible = self.list_height;
            self.selected = self.selected.saturating_sub(visible);
            self.scroll_offset = self.selected;
        }
    }

    fn move_page_down(&mut self) {
        if !self.stations.is_empty() {
            let visible = self.list_height;
            self.selected = (self.selected + visible).min(self.stations.len() - 1);
            if self.selected >= self.scroll_offset + visible {
                self.scroll_offset = self.selected.saturating_sub(visible) + 1;
            }
        }
    }

    fn move_home(&mut self) {
        if !self.stations.is_empty() {
            self.selected = 0;
            self.scroll_offset = 0;
        }
    }

    fn move_end(&mut self) {
        if !self.stations.is_empty() {
            self.selected = self.stations.len() - 1;
            self.scroll_offset = self.stations.len().saturating_sub(self.list_height);
        }
    }
}

struct Tui {
    terminal: Terminal<CrosstermBackend<io::Stdout>>,
}

impl Tui {
    fn new() -> Result<Self> {
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen)?;
        let backend = CrosstermBackend::new(stdout);

        Ok(Self {
            terminal: Terminal::new(backend)?,
        })
    }

    fn draw(&mut self, app: &mut App) -> Result<()> {
        self.terminal.draw(|f| ui(f, app))?;
        Ok(())
    }
}

impl Drop for Tui {
    fn drop(&mut self) {
        let _ = disable_raw_mode();
        let _ = execute!(self.terminal.backend_mut(), LeaveAlternateScreen);
        let _ = self.terminal.show_cursor();
    }
}

fn main() -> Result<()> {
    let mut app = App::new()?;
    let mut tui = Tui::new()?;

    loop {
        tui.draw(&mut app)?;

        if event::poll(std::time::Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Up => app.move_up(),
                        KeyCode::Down => app.move_down(),
                        KeyCode::PageUp => app.move_page_up(),
                        KeyCode::PageDown => app.move_page_down(),
                        KeyCode::Home => app.move_home(),
                        KeyCode::End => app.move_end(),
                        KeyCode::Enter => app.play_station(),
                        KeyCode::Char(' ') if app.playing_station.is_some() => {
                            app.toggle_playback()
                        }
                        KeyCode::Char('q') | KeyCode::Esc => {
                            app.stop();
                            break;
                        }
                        _ => {}
                    }
                }
            }
        }

        app.update_status();
    }

    Ok(())
}

fn load_stations() -> Result<Vec<Station>> {
    for path in playlist_candidates()? {
        if path.is_file() {
            return parse_m3u8(&path)
                .with_context(|| format!("failed to load playlist from {}", path.display()));
        }
    }

    parse_m3u8_str(EMBEDDED_PLAYLIST).context("failed to load embedded playlist")
}

fn playlist_candidates() -> Result<Vec<PathBuf>> {
    let mut candidates = Vec::new();

    if let Ok(current_dir) = std::env::current_dir() {
        candidates.push(current_dir.join("playlist/malaysia-radio.m3u8"));
    }

    let exe_path = std::env::current_exe().context("failed to resolve executable path")?;
    if let Some(exe_dir) = exe_path.parent() {
        candidates.push(exe_dir.join("playlist/malaysia-radio.m3u8"));

        if let Some(prefix_dir) = exe_dir.parent() {
            candidates.push(prefix_dir.join("share/my-radio-tui/malaysia-radio.m3u8"));
        }
    }

    candidates.push(PathBuf::from(
        "/usr/local/share/my-radio-tui/malaysia-radio.m3u8",
    ));

    candidates.dedup();
    Ok(candidates)
}

fn get_status_text(status: PlaybackStatus) -> &'static str {
    match status {
        PlaybackStatus::Stopped => "Stopped",
        PlaybackStatus::Loading => "Loading...",
        PlaybackStatus::Playing => "Streaming",
        PlaybackStatus::Paused => "Paused",
        PlaybackStatus::Error => "Error",
    }
}

fn get_status_color(status: PlaybackStatus) -> Color {
    match status {
        PlaybackStatus::Stopped => Color::DarkGray,
        PlaybackStatus::Loading => Color::Yellow,
        PlaybackStatus::Playing => Color::Green,
        PlaybackStatus::Paused => Color::LightYellow,
        PlaybackStatus::Error => Color::Red,
    }
}

fn ui(frame: &mut Frame, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(0),
            Constraint::Length(1),
            Constraint::Length(3),
        ])
        .split(frame.area());

    app.list_height = chunks[1].height.saturating_sub(2) as usize;

    let title = Paragraph::new(" Malaysia Radio TUI ")
        .bold()
        .centered()
        .style(Style::default().fg(Color::Cyan))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan)),
        );

    frame.render_widget(title, chunks[0]);

    let list_height = chunks[1].height as usize;
    let visible_stations: Vec<ListItem> = app
        .stations
        .iter()
        .skip(app.scroll_offset)
        .take(list_height.saturating_sub(2))
        .enumerate()
        .map(|(i, s)| {
            let actual_idx = app.scroll_offset + i;
            let style = if actual_idx == app.selected {
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::LightGreen)
                    .bold()
            } else {
                Style::default().fg(Color::White)
            };
            ListItem::new(s.name.clone()).style(style)
        })
        .collect();

    let list = List::new(visible_stations)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Stations ")
                .border_style(Style::default().fg(Color::DarkGray)),
        )
        .style(Style::default().fg(Color::White))
        .highlight_style(Style::default().fg(Color::Black).bg(Color::LightGreen));

    frame.render_widget(list, chunks[1]);

    let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
        .thumb_style(Style::default().fg(Color::Gray))
        .track_style(Style::default().fg(Color::DarkGray));

    let mut scroll_state = ScrollbarState::new(app.stations.len())
        .position(app.selected)
        .viewport_content_length(app.list_height);
    frame.render_stateful_widget(scrollbar, chunks[1], &mut scroll_state);

    let help_text = Paragraph::new(
        " ↑/↓ Navigate  Enter Play  Space Pause  q/ESC Quit  PgUp/PgDn Page  Home/End First/Last ",
    )
    .style(Style::default().fg(Color::DarkGray))
    .centered();
    frame.render_widget(help_text, chunks[2]);

    let now_playing = if let Some(name) = &app.playing_station {
        format!("Now: {}", name)
    } else {
        String::from("Stopped")
    };

    let status_text = get_status_text(app.status);
    let status_color = get_status_color(app.status);

    let status = Paragraph::new(format!(" {} | {}", status_text, now_playing))
        .style(Style::default().fg(status_color))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::DarkGray)),
        );

    frame.render_widget(status, chunks[3]);
}
