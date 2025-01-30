//! The Engine handles the core logic of applying operations to units,
//! including resolution of dependencies, building arguments, and
//! error reporting.
//!
//! It will notify observers with events as it progresses through unit
//! execution for UI, logging and telemetry purposes.

mod loader;
mod resolver;
mod unit_execution;
mod shell_executor;
mod runner;
mod executor_pool;

pub use resolver::ResolvableNode;

use crate::models::{UnitArc, Operation};
use crate::events::{Event, EventHandler, ObserverArc};

use tracing::instrument;
use std::{fmt, sync::Arc, collections::HashMap};

use loader::Loader;
use resolver::resolve;

use anyhow::Result;
use async_std::path::PathBuf;
use runner::Runner;

#[derive(Debug)]
pub struct Opts {
    pub remove_deps: bool,
    pub search_paths: Vec<PathBuf>,
    pub operation: Operation,
    pub unit: UnitArc,
    pub adapters: HashMap<String, String>,
}

#[derive(Clone)]
pub struct Context {
    pub opts: Arc<Opts>,
    pub ev_handler: EventHandler,
}

pub struct Engine {
    runner: Runner,
    ev_handler: EventHandler,
    opts: Arc<Opts>,
}

impl fmt::Debug for Engine {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Engine {{ opts: {:?} }}", self.opts)?;
        Ok(())
    }
}

impl Engine {
    pub fn new(opts: Opts, observers: Vec<ObserverArc>) -> Engine {
        let loader = Loader::from_search_paths(opts.search_paths.clone());
        let ev_handler = EventHandler::new(observers);
        let opts = Arc::new(opts);
        let ctx = Context {
            opts: opts.clone(),
            ev_handler: ev_handler.clone(),
        };
        let runner = Runner::new(loader, ctx);

        Engine {
            ev_handler,
            runner,
            opts,
        }
    }

    pub async fn run(&mut self) -> Result<()> {
        let op = self.opts.operation;
        let unit = self.opts.unit.clone();

        let result = match op {
            Operation::Check => self.run_unit(unit, op).await,
            Operation::Apply => self.run_with_dependencies(unit, op).await,
            Operation::Remove => {
                if self.opts.remove_deps {
                    self.run_with_dependencies(unit.clone(), op).await
                } else {
                    self.run_unit(unit, op).await
                }
            },
            _ => panic!("Operation {:?} can't be run directly", op),
        };

        self.runner.finalize().await?;

        let finalization_event = match result {
            Ok(_) => Event::EngineSuccess,
            Err(ref e) => Event::Error(format!("{:#}", e)),
        };

        self.ev_handler.handle(finalization_event)?;

        // Any errors have been sent to the event handler for display, so we can
        // return OK upstream
        Ok(())
    }

    async fn run_with_dependencies(&mut self, unit: UnitArc, op: Operation) -> Result<()> {
        self.ev_handler.handle(Event::Resolving)?;

        let ordered_units = resolve(unit, &mut self.runner).await?;

        self.ev_handler.handle(Event::Resolved(ordered_units.clone()))?;

        for unit in ordered_units {
            self.run_unit(unit, op).await?;
        }

        Ok(())
    }

    #[instrument]
    async fn run_unit(&mut self, unit: UnitArc, op: Operation) -> Result<()> {
        async fn do_op(runner: &mut Runner, unit: UnitArc, op: Operation) -> Result<()> {
            match op {
                Operation::Check => {
                    runner.check(unit).await?;
                },
                Operation::Apply => {
                    if ! runner.check(unit.clone()).await? {
                        runner.apply(unit.clone()).await?;
                    }
                }
                Operation::Remove => {
                    if runner.check(unit.clone()).await? {
                        runner.remove(unit.clone()).await?;
                    }
                }
                _ => panic!("Operation {:?} can't be run directly from the engine", op),
            };

            Ok(())
        }

        let result = do_op(&mut self.runner, unit.clone(), op).await;
        return result;
    }
}
