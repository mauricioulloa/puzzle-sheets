use anyhow::Result;
use clap::{Parser, Subcommand};
use puzzle_sheets::serve;

#[derive(Parser)]
#[command(
    name = "puzzle-sheets",
    version,
    about = "Printable puzzle worksheets for students and educators"
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Run the web app and MCP server
    Serve(serve::ServeArgs),
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "puzzle_sheets=info".into()),
        )
        .with_target(false)
        .init();

    match Cli::parse().command {
        Command::Serve(args) => serve::run(args).await,
    }
}
