use clap::Parser;
use logup::OutlogArgs;

// Single-thread on purpose to consume the least amount of resources.
#[tokio::main(flavor = "current_thread")]
async fn main() {
    let cli = OutlogArgs::parse();
    logup::run(cli).await;
}
