use clap::Parser;
use sommelier_auction_order_engine::{config::Config, engine::OrderEngine};
use tracing::{debug, error};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    config: String,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let args = Args::parse();
    if args.config.is_empty() {
        panic!("config file path is required");
    }

    let config: Config = confy::load_path(&args.config).expect("failed to load config");
    debug!("config: {config:?}");

    let mut engine = OrderEngine::new(config);
    if let Err(e) = engine.start().await {
        error!("error running engine: {e}");
    }
}
