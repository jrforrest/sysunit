/// Reports on events emitted from the engine to the CLI
///
/// There's some complexity here in that these events don't map very well
/// to the UI. Also, we need to support capturing unit output for future
/// printing in case an error arises, and change the output based on level
/// of verbosity.  
///
/// This is handled with nested state machines, called Contexts here. Contexts
/// are entered and exited by their parent in response to engine events. Events
/// are delegated down the context chain until they are handled.
use anyhow::Result;

use crate::events::{Event, Observer};

mod ctx;

use ctx::RootContext;

pub struct EngineLogger {
    context: RootContext,
}

/// Sets the noise level for a reporter
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

