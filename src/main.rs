mod app;
mod cli;
mod diff;
mod executor;
mod tui;

use clap::Parser;

#[tokio::main]
async fn main() {
    let args = cli::Args::parse();
    executor::run_loop(args, |result| {
        print!("{}", result.stdout);
        if !result.stderr.is_empty() {
            eprint!("{}", result.stderr);
        }
    })
    .await;
}
