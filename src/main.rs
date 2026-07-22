use clap::Parser;

fn main() {
    let args = scope::Args::parse();
    println!("{} every {}s", args.command.join(" "), args.interval);
}
