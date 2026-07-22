use clap::Parser;

#[derive(Parser)]
#[command(name = "scope", version)]
pub struct Args {
    /// Seconds between refreshes
    #[arg(short = 'n', long, default_value_t = 2.0)]
    pub interval: f64,

    /// The command to run
    #[arg(required = true, trailing_var_arg = true)]
    pub command: Vec<String>,
}
