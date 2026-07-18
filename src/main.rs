use clap::Parser;

#[derive(Parser)]
#[command(name = "scope", version)]
struct Args {
    /// Seconds between refreshes
    #[arg(short = 'n', long, default_value_t = 2.0)]
    interval: f64,

    /// The command to run
    #[arg(required = true, trailing_var_arg = true)]
    command: Vec<String>,
}

fn main() {
    let args = Args::parse();
    println!("Usage scope");
    println!("{} every {}s", args.command.join(" "), args.interval);
}
