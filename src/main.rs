use anyhow::Result;
use clap::Parser;
use puzzle_sheets::serve::{self, ServeArgs};

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "puzzle_sheets=info".into()),
        )
        .with_target(false)
        .init();

    serve::run(ServeArgs::parse()).await
}
