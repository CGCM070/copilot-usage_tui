mod api;
mod cache;
mod cli;
mod config;
mod models;
mod themes;
mod ui;
mod waybar;

use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    cli::run().await
}