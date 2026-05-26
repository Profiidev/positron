use backend::Cli;
use clap::Parser;

#[tokio::main]
async fn main() {
  Cli::parse().run().await;
}
