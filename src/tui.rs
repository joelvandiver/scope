use std::io::{self, stdout};
use std::panic;
use std::sync::{Arc, Mutex};
use std::time::Duration;

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

use crate::app::AppState;
use crate::cli::Args;

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
            let state = state.lock().unwrap();
            render(frame, &state, args);
        })?;

        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                match (key.code, key.modifiers) {
                    (KeyCode::Char('q'), _)
                    | (KeyCode::Char('c'), KeyModifiers::CONTROL) => {
                        cancel.cancel();
                        return Ok(());
                    }
                    _ => {}
                }
            }
        }
    }
}

fn render(frame: &mut ratatui::Frame, state: &AppState, args: &Args) {
    let area = frame.area();

    let (header_area, output_area) = if args.no_title {
        (None, area)
    } else {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(2), Constraint::Min(0)])
            .split(area);
        (Some(chunks[0]), chunks[1])
    };

    if let Some(ha) = header_area {
        render_header(frame, state, ha);
    }

    render_output(frame, state, output_area);
}

fn render_header(frame: &mut ratatui::Frame, state: &AppState, area: ratatui::layout::Rect) {
    let now = Local::now().format("%a %b %e %H:%M:%S %Y").to_string();
    let left = format!("Every {:.1}s: {}", state.interval, state.command);
    let right = format!("{}: {}", state.hostname, now);

    let width = area.width as usize;
    let padding = width.saturating_sub(left.len() + right.len());
    let header_text = format!("{}{}{}", left, " ".repeat(padding), right);

    let header = Paragraph::new(header_text)
        .style(Style::default().bg(Color::Blue).add_modifier(Modifier::BOLD))
        .block(Block::default().borders(Borders::BOTTOM));

    frame.render_widget(header, area);
}

fn render_output(frame: &mut ratatui::Frame, state: &AppState, area: ratatui::layout::Rect) {
    let lines: Vec<Line> = state
        .diff_lines
        .iter()
        .map(|dl| Line::from(Span::raw(dl.content.clone())))
        .collect();

    let paragraph = Paragraph::new(lines)
        .block(Block::default())
        .scroll((state.scroll_offset, 0));

    frame.render_widget(paragraph, area);
}
