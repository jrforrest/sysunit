//! Runs units via posix sh

use anyhow::{anyhow, Result};
use crate::engine::shell_executor::adapter::build_command;
use crate::{
    models::{Target, Operation, ValueSet, Dependency, Meta},
    events::{Event, OpEventHandler},
};

use std::sync::{Arc, Mutex};
use async_process::ChildStdout;

use super::Context as EngineContext;

mod subprocess;
mod stdout_data;
mod message_stream;
mod adapter;

use subprocess::Subprocess;
use message_stream::MessageStream;

const SHELL_SLUG: &str = include_str!("shell_slug.template.sh");

/// Executes a unit's shell script and provides an interface to interract with it
pub struct ShellExecutor {
    subprocess: Subprocess,
    ctx: EngineContext,
    msg_stream: MessageStream<ChildStdout>,
}

impl ShellExecutor {
    pub async fn init(
        target: &Target,
        ctx: EngineContext,
    ) -> Result<Self> {
        let command = build_command(target, &ctx.opts)?;
        let mut subprocess = Subprocess::init(command)?;
        let stdout_parser = stdout_data::StdoutDataProducer::new(subprocess.take_stdout());
        let msg_stream = MessageStream::new(stdout_parser);

        let mut executor = ShellExecutor {
            subprocess,
            ctx,
            msg_stream,
        };

        executor.send_stdin(SHELL_SLUG).await?;

        Ok(executor)
    }

    pub async fn get_meta(&mut self, op_ev_handler: OpEventHandler, script: &str, args: &ValueSet) -> Result<Meta> {
        self.run_op(Operation::Meta, script, args).await?;
        self.msg_stream.get_meta(op_ev_handler).await
    }
    pub async fn get_deps(&mut self, op_ev_handler: OpEventHandler, script: &str, args: &ValueSet) -> Result<Vec<Dependency>> {
        self.run_op(Operation::Deps, script, args).await?;
        self.msg_stream.get_deps(op_ev_handler).await
    }

    pub async fn check(&mut self, op_ev_handler: OpEventHandler, script: &str, args: &ValueSet) -> Result<(bool, ValueSet)> {
        self.run_op(Operation::Check, script, args).await?;
        self.msg_stream.get_check_values(op_ev_handler).await
    }

    pub async fn apply(&mut self, op_ev_handler: OpEventHandler, script: &str, args: &ValueSet) -> Result<ValueSet> {
        self.run_op(Operation::Apply, script, args).await?;
        self.msg_stream.get_apply_values(op_ev_handler).await
    }

    pub async fn remove(&mut self, op_ev_handler: OpEventHandler, script: &str, args: &ValueSet) -> Result<ValueSet> {
        self.run_op(Operation::Remove, script, args).await?;
        self.msg_stream.get_remove_values(op_ev_handler).await
    }

    /// Runs an operation from the given script
    async fn run_op(&mut self, op: Operation, script: &str, args: &ValueSet) -> Result<()> {
        let argstr = args_str(args);
        self.send_stdin(&format!("(\n{}\n{}\n{}\n)\n _emit status $? \n", script, argstr, op.to_string())).await?;
        Ok(())
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

fn args_str(args: &ValueSet) -> String {
    let mut argstr = String::new();
    for (key, value) in args.values.iter() {
        argstr.push_str(&format!("{}=\"{}\"\n", key, value));
    }
    argstr
}
