use backend::Cli;
use clap::Parser;
#[cfg(debug_assertions)]
use dotenvy::dotenv;

#[tokio::main]
async fn main() {
  #[cfg(debug_assertions)]
  dotenv().ok();

  Cli::parse().run().await;
}
