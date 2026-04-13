mod parser;
mod player;

use anyhow::Result;
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use parser::{parse_m3u8, Station};
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

struct App {
    stations: Vec<Station>,
    selected: usize,
    scroll_offset: usize,
    playing_station: Option<String>,
    status: PlaybackStatus,
    player: Player,
}

impl App {
    fn new() -> Result<Self> {
        let playlist_path = std::env::current_dir()?.join("playlist/malaysia-radio.m3u8");
        if !playlist_path.exists() {
            let installed_path = PathBuf::from("/usr/local/bin/playlist/malaysia-radio.m3u8");
            if installed_path.exists() {
                return Self::new_with_path(installed_path);
            }
        }
        let stations = parse_m3u8(&playlist_path)?;
        let player = Player::new()?;

        Ok(Self {
            stations,
            selected: 0,
            scroll_offset: 0,
            playing_station: None,
            status: PlaybackStatus::Stopped,
            player,
        })
    }

    fn new_with_path(playlist_path: PathBuf) -> Result<Self> {
        let stations = parse_m3u8(&playlist_path)?;
        let player = Player::new()?;

        Ok(Self {
            stations,
            selected: 0,
            scroll_offset: 0,
            playing_station: None,
            status: PlaybackStatus::Stopped,
            player,
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
            let visible = 20;
            if self.selected >= self.scroll_offset + visible {
                self.scroll_offset = self.selected.saturating_sub(visible) + 1;
            }
        }
    }

    fn move_page_up(&mut self) {
        if !self.stations.is_empty() {
            let visible = 20;
            self.selected = self.selected.saturating_sub(visible);
            self.scroll_offset = self.selected;
        }
    }

    fn move_page_down(&mut self) {
        if !self.stations.is_empty() {
            let visible = 20;
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
            let visible = 20;
            self.scroll_offset = self.stations.len().saturating_sub(visible);
        }
    }
}

fn main() -> Result<()> {
    let mut app = App::new()?;

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    loop {
        terminal.draw(|f| ui(f, &mut app))?;

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
                        KeyCode::Char(' ') => {
                            if app.playing_station.is_some() {
                                app.toggle_playback();
                            }
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

    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    disable_raw_mode()?;
    terminal.show_cursor()?;

    Ok(())
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

    let title = Paragraph::new(" Radio TUI ")
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

    let mut scroll_state = ScrollbarState::new(app.stations.len()).position(app.selected);
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
