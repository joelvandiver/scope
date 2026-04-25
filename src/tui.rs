use std::io::{self, stdout};
use std::panic;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use ansi_to_tui::IntoText;
use chrono::Local;
use crossterm::event::{self, Event, KeyCode, KeyModifiers};
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use crossterm::ExecutableCommand;
use ratatui::backend::CrosstermBackend;
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Terminal;
use tokio_util::sync::CancellationToken;
use unicode_width::UnicodeWidthStr;

use crate::app::AppState;
use crate::cli::Args;
use crate::diff::{DiffLine, DiffLineKind};

pub fn install_panic_hook() {
    let original = panic::take_hook();
    panic::set_hook(Box::new(move |info| {
        let _ = restore_terminal();
        original(info);
    }));
}

pub fn init_terminal() -> io::Result<Terminal<CrosstermBackend<io::Stdout>>> {
    enable_raw_mode()?;
    stdout().execute(EnterAlternateScreen)?;
    Terminal::new(CrosstermBackend::new(stdout()))
}

pub fn restore_terminal() -> io::Result<()> {
    disable_raw_mode()?;
    stdout().execute(LeaveAlternateScreen)?;
    Ok(())
}

pub fn run(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    state: Arc<Mutex<AppState>>,
    args: &Args,
    cancel: CancellationToken,
) -> io::Result<()> {
    loop {
        terminal.draw(|frame| {
            let mut state = state.lock().unwrap();
            render(frame, &mut state, args);
        })?;

        if cancel.is_cancelled() {
            return Ok(());
        }

        if event::poll(Duration::from_millis(100))? {
            match event::read()? {
                Event::Key(key) => {
                    let mut state = state.lock().unwrap();
                    match (key.code, key.modifiers) {
                        (KeyCode::Char('q'), _)
                        | (KeyCode::Char('c'), KeyModifiers::CONTROL) => {
                            cancel.cancel();
                            return Ok(());
                        }
                        (KeyCode::Char('j'), _) | (KeyCode::Down, _) => state.scroll_down(1),
                        (KeyCode::Char('k'), _) | (KeyCode::Up, _) => state.scroll_up(1),
                        (KeyCode::Char('d'), _) => {
                            let h = state.viewport_height / 2;
                            state.scroll_down(h);
                        }
                        (KeyCode::Char('u'), _) => {
                            let h = state.viewport_height / 2;
                            state.scroll_up(h);
                        }
                        (KeyCode::Char('g'), _) | (KeyCode::Home, _) => state.scroll_top(),
                        (KeyCode::Char('G'), _) | (KeyCode::End, _) => state.scroll_bottom(),
                        _ => {}
                    }
                }
                Event::Resize(_, _) => {
                    terminal.autoresize()?;
                    let mut state = state.lock().unwrap();
                    let max = state.max_scroll();
                    if state.scroll_offset > max {
                        state.scroll_offset = max;
                    }
                }
                _ => {}
            }
        }
    }
}

fn render(frame: &mut ratatui::Frame, state: &mut AppState, args: &Args) {
    let area = frame.area();

    let constraints = if args.no_title {
        vec![Constraint::Min(0), Constraint::Length(1)]
    } else {
        vec![Constraint::Length(2), Constraint::Min(0), Constraint::Length(1)]
    };

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(constraints)
        .split(area);

    let (header_area, output_area, status_area) = if args.no_title {
        (None, chunks[0], chunks[1])
    } else {
        (Some(chunks[0]), chunks[1], chunks[2])
    };

    if let Some(ha) = header_area {
        render_header(frame, state, ha);
    }

    state.viewport_height = output_area.height;
    render_output(frame, state, args, output_area);
    render_status(frame, state, status_area);
}

fn render_header(frame: &mut ratatui::Frame, state: &AppState, area: ratatui::layout::Rect) {
    let now = Local::now().format("%a %b %e %H:%M:%S %Y").to_string();
    let left = format!("Every {:.1}s: {}", state.interval, state.command);
    let right = format!("{}: {}", state.hostname, now);

    let width = area.width as usize;
    let padding = width.saturating_sub(left.width() + right.width());
    let header_text = format!("{}{}{}", left, " ".repeat(padding), right);

    let header = Paragraph::new(header_text)
        .style(Style::default().bg(Color::Blue).add_modifier(Modifier::BOLD))
        .block(Block::default().borders(Borders::BOTTOM));

    frame.render_widget(header, area);
}

fn render_status(frame: &mut ratatui::Frame, state: &AppState, area: ratatui::layout::Rect) {
    let dim = Style::default().fg(Color::DarkGray);
    let bold = Style::default().add_modifier(Modifier::BOLD);

    // Left side: run counter + exit code
    let mut left_spans = vec![
        Span::styled("  run ", dim),
        Span::styled(format!("#{}", state.run_count), bold),
    ];

    if let Some(code) = state.exit_code {
        let (label, style) = if code == 0 {
            ("  ok", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD))
        } else {
            ("  err", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD))
        };
        left_spans.push(Span::styled(label, style));
        if code != 0 {
            left_spans.push(Span::styled(format!(" ({})", code), Style::default().fg(Color::Red)));
        }
    }

    // Right side: scroll state
    let scroll_indicator = if state.run_count == 0 {
        String::new()
    } else if state.auto_scroll {
        "tracking ↓  ".to_string()
    } else {
        let pct = if state.max_scroll() == 0 {
            100
        } else {
            (state.scroll_offset as usize * 100) / state.max_scroll() as usize
        };
        format!("{}%  ", pct)
    };

    let left_line = Line::from(left_spans);
    let right_line = Line::from(Span::styled(scroll_indicator, dim));

    let status_style = Style::default().bg(Color::Rgb(30, 30, 30));

    // Render left-aligned
    frame.render_widget(
        Paragraph::new(left_line)
            .style(status_style)
            .block(Block::default().borders(Borders::TOP)),
        area,
    );
    // Render right-aligned on top (same area)
    frame.render_widget(
        Paragraph::new(right_line)
            .style(status_style)
            .alignment(ratatui::layout::Alignment::Right)
            .block(Block::default().borders(Borders::TOP)),
        area,
    );
}

fn diff_line_to_tui<'a>(dl: &'a DiffLine, args: &Args) -> Line<'a> {
    // Diff highlighting takes priority over ANSI color for changed lines.
    if args.differences {
        let style = match dl.kind {
            DiffLineKind::Added => Style::default().fg(Color::Black).bg(Color::Green),
            DiffLineKind::Removed => Style::default().fg(Color::Black).bg(Color::Red),
            DiffLineKind::Same => Style::default(),
        };
        if style != Style::default() {
            return Line::from(Span::styled(dl.content.clone(), style));
        }
    }

    // For unchanged lines (or when -d is off), parse ANSI codes if -c is set.
    if args.color {
        if let Ok(text) = dl.content.as_str().into_text() {
            if let Some(line) = text.lines.into_iter().next() {
                return line;
            }
        }
    }

    Line::from(Span::raw(dl.content.clone()))
}

fn render_output(
    frame: &mut ratatui::Frame,
    state: &AppState,
    args: &Args,
    area: ratatui::layout::Rect,
) {
    let lines: Vec<Line> = if let Some(err) = &state.error {
        vec![Line::from(Span::styled(
            err.clone(),
            Style::default().fg(Color::Red),
        ))]
    } else if state.run_count == 0 {
        vec![Line::from(Span::styled(
            "Waiting for first run…",
            Style::default().fg(Color::DarkGray),
        ))]
    } else {
        state
            .diff_lines
            .iter()
            .map(|dl| diff_line_to_tui(dl, args))
            .collect()
    };

    let paragraph = Paragraph::new(lines)
        .block(Block::default())
        .scroll((state.scroll_offset, 0));

    frame.render_widget(paragraph, area);
}
