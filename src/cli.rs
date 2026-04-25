use clap::Parser;

#[derive(Parser, Debug, Clone)]
#[command(name = "scope", about = "A terminal UI replacement for the Linux watch command")]
pub struct Args {
    /// Seconds to wait between updates
    #[arg(short = 'n', long, default_value = "2.0", value_name = "seconds")]
    pub interval: f64,

    /// Hide the header
    #[arg(short = 't', long)]
    pub no_title: bool,

    /// Interpret ANSI color codes in command output
    #[arg(short = 'c', long)]
    pub color: bool,

    /// Exit if the command has a non-zero exit code
    #[arg(short = 'e', long)]
    pub errexit: bool,

    /// Pass command to exec directly instead of a shell
    #[arg(short = 'x', long)]
    pub exec: bool,

    /// Precise timing: subtract command runtime from interval
    #[arg(short = 'p', long)]
    pub precise: bool,

    /// Command to run
    #[arg(required = true, trailing_var_arg = true, allow_hyphen_values = true)]
    pub command: Vec<String>,
}
