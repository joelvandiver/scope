mod app;
mod cli;
mod diff;
mod executor;
mod tui;

use clap::Parser;
use tokio_util::sync::CancellationToken;

#[tokio::main]
async fn main() {
    let args = cli::Args::parse();

    let hostname = std::fs::read_to_string("/proc/sys/kernel/hostname")
        .unwrap_or_default()
        .trim()
        .to_string();

    tui::install_panic_hook();
    let mut terminal = tui::init_terminal().expect("failed to initialize terminal");

    let command = args.command.join(" ");
    let state = app::AppState::new(command, args.interval, hostname);
    let cancel = CancellationToken::new();

    let exec_state = state.clone();
    let exec_cancel = cancel.clone();
    let exec_args = args.clone();
    tokio::spawn(async move {
        executor::run_loop(exec_args, exec_state, exec_cancel).await;
    });

    let result = tui::run(&mut terminal, state, &args, cancel);

    tui::restore_terminal().expect("failed to restore terminal");
    result.expect("tui error");
}
