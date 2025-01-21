//! Runs units via posix sh

use tracing::instrument;
use anyhow::{anyhow, Result>;
use crate::engine::shell_executor::adapter::build_command;
use std::fmt;
use crate::{
    models::{UnitArc, Operation, OpCompletion, ValueSet},
    events::{Event, OpEventHandler},
};

use super::Context as EngineContext;

mod subprocess;
mod stdout_data;
mod message_stream;
mod adapter;

use subprocess::Subprocess;
use message_stream::MessageStream;

const SHELL_SLUG: &str = include_str!("shell_slug.template.sh");

pub struct ShellExecutor {
    subprocess: Subprocess,
    unit: UnitArc,
    ctx: EngineContext,
}

impl fmt::Debug for ShellExecutor {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "ShellExecutor {{ unit: {:?} }}", self.unit)
    }
}

/// Executes a unit's shell script and provides an interface to interract with it
impl ShellExecutor {
    pub async fn init(
        unit: UnitArc,
        script: &str,
        ctx: EngineContext,
    ) -> Result<Self> {
        let command = build_command(unit.clone(), &ctx.opts)?;
        let subprocess = Subprocess::init(command)?;

        let mut executor = ShellExecutor {
            subprocess,
            ctx,
            unit: unit.clone(),
        };

        executor.send_stdin(&format!("UNIT_NAME={}\n", unit.name)).await?;
        executor.send_stdin(SHELL_SLUG).await?;
        executor.send_stdin(script).await?;

        Ok(executor)
    }

    /// Set arguments for the unit
    pub async fn set_args(&mut self, args: ValueSet) -> Result<()> {
        for (key, value) in args.values.iter() {
            self.send_stdin(&format!("{}=\"{}\"\n", key, value)).await?
        }
        Ok(())
    }

    /// Run an operation on the unit
    pub async fn run_op(&mut self, op: Operation, op_ev_handler: OpEventHandler) -> Result<OpCompletion> {
        self.run_command(&op.to_string()).await?;
        let mut stdout_parser = stdout_data::StdoutDataProducer::new(self.subprocess.get_stdout());
        let mut message_stream = MessageStream::new(&mut stdout_parser, op_ev_handler.clone());

        let opc = match op {
            Operation::Meta => message_stream.get_meta().await,
            Operation::Deps => message_stream.get_deps().await,
            Operation::Check => message_stream.get_check_values().await,
            Operation::Apply => message_stream.get_apply_values().await,
            Operation::Remove => message_stream.get_remove_values().await,
        }?;

        Ok(opc)
    }

    #[instrument]
    /// Close pipes to the unit and wait for it to exit
    pub async fn finalize(mut self) -> Result<()> {
        use async_std::io::ReadExt;
        let mut stderr = String::new();
        self.subprocess.get_stderr().read_to_string(&mut stderr).await?;

        let status_code = self.subprocess.finalize().await?;

        if status_code != 0 {
            return Err(anyhow!("adapter exited with status code {} {}", status_code, stderr));
        }

        self.ctx.ev_handler.handle(Event::Debug(format!("Script {} exited", self.unit.name)))?;
        Ok(())
    }

    async fn run_script(&mut self, command: &str) -> Result<()> {
        self.send_stdin(&format!("( {} )\n _emit status $? \n", command)).await
    }

    async fn send_stdin(&mut self, data: &str) -> Result<()> {
        self.subprocess.write_stdin(data).await
    }
}
