//! Runs units via posix sh

use anyhow::{anyhow, Result};
use crate::engine::shell_executor::adapter::build_command;
use crate::{
    models::{Target, Operation, OpCompletion, ValueSet},
    events::{Event, OpEventHandler},
};

use std::sync::{Arc, Mutex};

use super::Context as EngineContext;

mod subprocess;
mod stdout_data;
mod message_stream;
mod adapter;

use subprocess::Subprocess;
use message_stream::MessageStream;

const SHELL_SLUG: &str = include_str!("shell_slug.template.sh");

pub type ExecutorArc = Arc<Mutex<ShellExecutor>>;

/// Executes a unit's shell script and provides an interface to interract with it
pub struct ShellExecutor {
    subprocess: Subprocess,
    ctx: EngineContext,
}

impl ShellExecutor {
    pub async fn init(
        target: &Target,
        ctx: EngineContext,
    ) -> Result<Self> {
        let command = build_command(target, &ctx.opts)?;
        let subprocess = Subprocess::init(command)?;

        let mut executor = ShellExecutor {
            subprocess,
            ctx,
        };

        executor.send_stdin(SHELL_SLUG).await?;

        Ok(executor)
    }


    /// Runs an operation from the given script
    pub async fn run_op(&mut self, op: Operation, op_ev_handler: OpEventHandler, script: &str, args: Arc<Mutex<ValueSet>>) -> Result<OpCompletion> {
        let argstr = args_str(args);
        self.send_stdin(&format!("(\n{}\n{}\n{}\n)\n _emit status $? \n", script, argstr, op.to_string())).await?;
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

    /// Close pipes to the shell subprocess and wait for it to exit
    pub async fn finalize(mut self) -> Result<()> {
        use async_std::io::ReadExt;
        let mut stderr = String::new();
        let subprocess_string = format!("{:?}", &self.subprocess);
        self.subprocess.get_stderr().read_to_string(&mut stderr).await?;

        let status_code = self.subprocess.finalize().await?;

        if status_code != 0 {
            return Err(anyhow!("adapter exited with status code {} {}", status_code, stderr));
        }

        self.ctx.ev_handler.handle(Event::Debug(format!("Executor {:?} exited", subprocess_string)))?;
        Ok(())
    }

    async fn send_stdin(&mut self, data: &str) -> Result<()> {
        self.subprocess.write_stdin(data).await
    }
}

fn args_str(args: Arc<Mutex<ValueSet>>) -> String {
    let args = args.lock().unwrap();
    let mut argstr = String::new();
    for (key, value) in args.values.iter() {
        argstr.push_str(&format!("{}=\"{}\"\n", key, value));
    }
    argstr
}
