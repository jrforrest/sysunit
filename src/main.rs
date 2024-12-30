mod cli;
mod engine;
mod models;
mod util;
mod parser;
mod events;

use async_std::task;
use anyhow::Result;
use tracing_subscriber::{registry::Registry, prelude::*, EnvFilter};
use tracing_tree::HierarchicalLayer;
use std::sync::{Arc, Mutex};

use crate::engine::Engine;
use crate::cli::{CLI, EngineLogger};

fn main() -> Result<()> {
    task::block_on(run())
}

async fn run() -> Result<()> {
    let subscriber = Registry::default()
        .with(EnvFilter::from_default_env())
        .with(HierarchicalLayer::new(2));
    tracing::subscriber::set_global_default(subscriber).unwrap();

    let cli = CLI::init()?;
    let engine_observer: Arc<Mutex<EngineLogger>> = Arc::new(Mutex::new(cli.get_engine_observer()?));
    let mut engine = Engine::new(cli.get_engine_options()?, vec!(engine_observer));

    engine.run().await
}
