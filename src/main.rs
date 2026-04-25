mod app;
mod cli;
mod diff;
mod executor;
mod tui;

use clap::Parser;

fn main() {
    let args = cli::Args::parse();
    println!("{args:?}");
}
