use std::io::{self, Stdout};

use crossterm::event::KeyCode;
use crossterm::execute;
use crossterm::terminal::{
    self, EnterAlternateScreen, LeaveAlternateScreen,
};
use ratatui::backend::CrosstermBackend;
use ratatui::layout::{Alignment, Constraint, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Terminal;

use crate::config::APP_CONFIG;
use crate::state::UiState;

const MIN_COLS: u16 = 60;
const MIN_ROWS: u16 = 18;
const KEY_WIDTH: u16 = 6;

pub struct Ui {
    terminal: Terminal<CrosstermBackend<Stdout>>,
}

impl Ui {
    pub fn new() -> Result<Self, String> {
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen)
            .map_err(|e| format!("failed to enter alternate screen: {e}"))?;
        terminal::enable_raw_mode()
            .map_err(|e| format!("failed to enable raw mode: {e}"))?;

        let backend = CrosstermBackend::new(stdout);
        let terminal = Terminal::new(backend)
            .map_err(|e| format!("failed to create terminal: {e}"))?;

        Ok(Self { terminal })
    }

    pub fn draw(&mut self, state: &UiState) -> Result<(), String> {
        self.terminal
            .draw(|frame| {
                let area = frame.area();

                // Check minimum size
                if area.width < MIN_COLS || area.height < MIN_ROWS {
                    let msg = Paragraph::new("Please resize terminal to at least 60x18")
                        .alignment(Alignment::Center)
                        .style(Style::default().fg(Color::Red));
                    frame.render_widget(msg, area);
                    return;
                }

                // Vertical layout: header, body, status bar
                let chunks = Layout::vertical([
                    Constraint::Length(1), // header
                    Constraint::Min(0),    // body (piano keys)
                    Constraint::Length(1), // status bar
                ])
                .split(area);

                render_header(frame, chunks[0], state);
                render_piano(frame, chunks[1], state);
                render_status(frame, chunks[2], state);
            })
            .map_err(|e| format!("render error: {e}"))?;
        Ok(())
    }

    pub fn cleanup(&mut self) {
        let _ = terminal::disable_raw_mode();
        let _ = execute!(self.terminal.backend_mut(), LeaveAlternateScreen);
        let _ = self.terminal.show_cursor();
    }
}

// ── Header ──────────────────────────────────────────────────────

fn render_header(frame: &mut ratatui::Frame, area: Rect, state: &UiState) {
    let voice_color = match state.voice_count {
        0..=5 => Color::Green,
        6..=7 => Color::Yellow,
        _ => Color::Red,
    };

    let title = Span::styled(
        " Terminal Piano ",
        Style::default()
            .fg(Color::White)
            .add_modifier(Modifier::BOLD),
    );
    let voices = Span::styled(
        format!("Voices: {}/8 ", state.voice_count),
        Style::default().fg(voice_color),
    );
    let quit = Span::styled(
        "[Esc] Quit ",
        Style::default().fg(Color::DarkGray),
    );

    // Left-align title, right-align voices + quit
    let right_len = voices.width() + quit.width();
    let pad = area
        .width
        .saturating_sub(title.width() as u16 + right_len as u16);

    let header = Line::from(vec![
        title,
        Span::raw(" ".repeat(pad as usize)),
        voices,
        quit,
    ]);

    let para = Paragraph::new(header)
        .style(Style::default().bg(Color::Blue));
    frame.render_widget(para, area);
}

// ── Piano keys ──────────────────────────────────────────────────

fn render_piano(frame: &mut ratatui::Frame, area: Rect, state: &UiState) {
    let notes = APP_CONFIG.notes;
    let mid = notes.len() / 2;

    // Each key row is 6 lines tall (border top, key char, solfege, note, border bottom, gap)
    // Two rows + labels + spacing
    let row_height: u16 = 5;

    let chunks = Layout::vertical([
        Constraint::Min(0),          // top padding
        Constraint::Length(row_height), // octave 5 (upper row: Q W E R T Y U)
        Constraint::Length(1),       // gap
        Constraint::Length(row_height), // octave 4 (lower row: A S D F G H J)
        Constraint::Min(0),          // bottom padding
    ])
    .split(area);

    render_key_row(frame, chunks[1], &notes[mid..], mid, state, "Octave 5");
    render_key_row(frame, chunks[3], &notes[..mid], 0, state, "Octave 4");
}

fn render_key_row(
    frame: &mut ratatui::Frame,
    area: Rect,
    notes: &[crate::config::NoteConfig],
    index_offset: usize,
    state: &UiState,
    label: &str,
) {
    let key_count = notes.len() as u16;
    let total_keys_width = key_count * KEY_WIDTH + (key_count - 1); // keys + gaps
    let label_width = label.len() as u16 + 2; // with padding

    // Horizontal layout: left pad, keys, gap, label, right pad
    let left_pad = area.width.saturating_sub(total_keys_width + label_width + 2) / 2;

    // Build constraints for each key + 1-col gap between keys
    let mut constraints: Vec<Constraint> = Vec::new();
    constraints.push(Constraint::Length(left_pad));
    for i in 0..key_count {
        constraints.push(Constraint::Length(KEY_WIDTH));
        if i < key_count - 1 {
            constraints.push(Constraint::Length(1)); // gap
        }
    }
    constraints.push(Constraint::Length(2)); // gap before label
    constraints.push(Constraint::Length(label_width));
    constraints.push(Constraint::Min(0)); // right pad

    let cols = Layout::horizontal(constraints).split(area);

    // Render each key
    for (i, note) in notes.iter().enumerate() {
        let col_idx = 1 + i * 2; // skip left_pad, account for gaps
        let active = state.is_active(index_offset + i);
        render_single_key(frame, cols[col_idx], note, active);
    }

    // Render octave label
    let label_idx = 1 + (key_count as usize) * 2; // after all keys + gaps + gap-before-label
    let label_para = Paragraph::new(label)
        .alignment(Alignment::Left)
        .style(Style::default().fg(Color::Yellow).add_modifier(Modifier::DIM));
    // Vertically center the label
    let label_area = Rect {
        y: cols[label_idx].y + cols[label_idx].height / 2,
        height: 1,
        ..cols[label_idx]
    };
    frame.render_widget(label_para, label_area);
}

fn render_single_key(
    frame: &mut ratatui::Frame,
    area: Rect,
    note: &crate::config::NoteConfig,
    active: bool,
) {
    let (bg, fg, mods) = if active {
        (Color::Cyan, Color::White, Modifier::BOLD)
    } else {
        (Color::DarkGray, Color::White, Modifier::empty())
    };

    let key_char = match note.key {
        KeyCode::Char(c) => c.to_ascii_uppercase().to_string(),
        _ => "?".into(),
    };

    // Extract solfege and note name from display like "Do (C4)"
    let (solfege, note_name) = note
        .display
        .split_once(' ')
        .map(|(s, rest)| {
            let trimmed = rest.trim_start_matches('(').trim_end_matches(')');
            (s, trimmed)
        })
        .unwrap_or((note.display, ""));

    let text = vec![
        Line::from(Span::styled(
            format!(" {} ", key_char),
            Style::default().fg(fg).add_modifier(mods),
        )),
        Line::from(Span::styled(
            format!("{:^w$}", solfege, w = KEY_WIDTH as usize - 2),
            Style::default().fg(if active { Color::White } else { Color::Gray }),
        )),
        Line::from(Span::styled(
            format!("{:^w$}", note_name, w = KEY_WIDTH as usize - 2),
            Style::default().fg(if active { Color::White } else { Color::DarkGray }),
        )),
    ];

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(if active { Color::Cyan } else { Color::Gray }))
        .style(Style::default().bg(bg));

    let para = Paragraph::new(text)
        .block(block)
        .alignment(Alignment::Center);

    frame.render_widget(para, area);
}

// ── Status bar ──────────────────────────────────────────────────

fn render_status(frame: &mut ratatui::Frame, area: Rect, state: &UiState) {
    let text = match state.last_note {
        Some(note) => format!(
            " Now playing: {} ",
            note,
        ),
        None => " Press a key to play a note ".to_string(),
    };

    let para = Paragraph::new(Span::styled(
        text,
        Style::default().fg(Color::White),
    ))
    .style(Style::default().bg(Color::Blue));

    frame.render_widget(para, area);
}
