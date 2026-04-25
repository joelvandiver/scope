mod app;
mod cli;
mod diff;
mod executor;
mod tui;

use clap::Parser;

#[tokio::main]
async fn main() {
    let args = cli::Args::parse();

    tui::install_panic_hook();
    let mut terminal = tui::init_terminal().expect("failed to initialize terminal");

    let command = args.command.join(" ");
    let state = app::AppState::new(command, args.interval);

    let result = tui::run(&mut terminal, state);

    tui::restore_terminal().expect("failed to restore terminal");
    result.expect("tui error");
}
