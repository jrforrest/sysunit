use anyhow::Result;

use crate::events::{Event, Observer};

mod ctx;

use ctx::RootContext;

pub struct EngineLogger {
    context: RootContext,
}

/// Sets the noise level for the CLI reporter
#[derive(Clone, Debug, PartialOrd, PartialEq)]
pub enum Verbosity {
    Quiet,
    Default,
    Verbose,
    Debug,
}

impl EngineLogger {
    pub fn new(verbosity: Verbosity) -> Self {
        Self { context: RootContext::new(verbosity) }
    }
}

impl Observer for EngineLogger {
    fn handle(&mut self, event: Event) -> Result<()> {
        self.context.handle(event);
        Ok(())
    }
}

