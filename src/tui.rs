use std::io::{self, stdout};
use std::panic;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use crossterm::event::{self, Event, KeyCode, KeyModifiers};
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use crossterm::ExecutableCommand;
use ratatui::backend::CrosstermBackend;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Paragraph};
use ratatui::Terminal;
use tokio_util::sync::CancellationToken;

use crate::app::AppState;

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
    cancel: CancellationToken,
) -> io::Result<()> {
    loop {
        terminal.draw(|frame| {
            let state = state.lock().unwrap();
            render(frame, &state);
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

fn render(frame: &mut ratatui::Frame, state: &AppState) {
    let area = frame.area();

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
