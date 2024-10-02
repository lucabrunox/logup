use clap::Parser;
use logup::LogupArgs;

// Single-thread on purpose to consume the least amount of resources.
#[tokio::main(flavor = "current_thread")]
async fn main() {
    let cli = LogupArgs::parse();
    logup::run(cli).await;
}
