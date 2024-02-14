use clap::Parser;
use tracing::{debug, error};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    config: String,
}

fn main() {
    tracing_subscriber::fmt::init();

    let args = Args::parse();
    if args.config.is_empty() {
        panic!("config file path is required");
    }
}
