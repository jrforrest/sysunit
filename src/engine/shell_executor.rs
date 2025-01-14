//! Runs units via posix SH

use tracing::instrument;
use std::fmt;
use crate::{
    models::{UnitArc, Operation, OpCompletion, ValueSet},
    events::{Event, EventHandler, OpEventHandler},
};

use std::collections::HashMap;
use anyhow::{Result, anyhow};

mod subprocess;
mod stdout_data;
mod message_stream;

use subprocess::{Command, Subprocess};
use message_stream::MessageStream;

const SHELL_SLUG: &str = include_str!("shell_slug.template.sh");

const SHELL_OPTS: [&str; 3] = ["-e", "-x", "-u"];

pub struct ShellExecutor {
    subprocess: Subprocess,
    unit: UnitArc,
    ev_handler: EventHandler,
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
        ev_handler: EventHandler,
    ) -> Result<Self> {
        let command = build_command(unit.clone())?;
        let subprocess = Subprocess::init(command)?;

        let mut executor = ShellExecutor {
            subprocess,
            ev_handler,
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
    pub async fn finalize(self) -> Result<()> {
        let status_code = self.subprocess.finalize().await?;

        if status_code != 0 {
            return Err(anyhow!("Shell script for unit {} exited with status code {}", self.unit.name, status_code));
        }

        self.ev_handler.handle(Event::Debug(format!("Script {} exited", self.unit.name)))?;
        Ok(())
    }

    async fn run_command(&mut self, command: &str) -> Result<()> {
        self.send_stdin(&format!("( {} )\n _emit status $? \n", command)).await
    }

    async fn send_stdin(&mut self, data: &str) -> Result<()> {
        self.subprocess.write_stdin(data).await
    }
}

fn build_command(unit: UnitArc) -> Result<Command> {
    let command = match unit.target {
        Some(ref target) => {
            if target.host != "localhost" {
                return Err(anyhow!("Can't run on target: {}.  Remote targets not supported yet", target));
            }

            Command {
                cmd: "su".into(),
                args: vec!["-".into(), target.user.clone(), "-c".into(), "/bin/sh".into()],
                env: HashMap::new(),
            }
        },
        None => Command {
            cmd: "/bin/sh".into(),
            args: SHELL_OPTS.iter().map(|s| s.to_string()).collect(),
            env: HashMap::new(),
        }
    };

    Ok(command)
}
