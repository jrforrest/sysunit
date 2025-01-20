//! Async execution and communication with child processes
use std::collections::HashMap;

use anyhow::{Result, Context};

use async_process::{Command as AsyncCommand, Child, Stdio, ChildStdout, ChildStderr};
use futures::AsyncWriteExt;
use std::fmt;

#[derive(Debug)]
pub struct Command {
    pub cmd: String,
    pub args: Vec<String>,
    pub env: HashMap<String, String>,
}

impl fmt::Display for Command {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} {}", self.cmd, self.args.join(" "))
    }
}

#[derive(Debug)]
pub struct Subprocess {
    child: Child,
    command: Command,
}

impl Subprocess {
    pub fn init(cmd: Command) -> Result<Self> {
        let child = AsyncCommand::new(&cmd.cmd)
            .args(&cmd.args)
            .envs(&cmd.env)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .context("Could not spawn subprocess")?;

        Ok(Self {
            child,
            command: cmd,
        })
    }

    /// Close all IO and wait for the subprocess to exit
    pub async fn finalize(mut self) -> Result<i32> {
        let status = self.child.status().await?;
        Ok(status.code().unwrap())
    }

    pub fn take_stdout(&mut self) -> ChildStdout {
        self.child.stdout.take().unwrap()
    }

    pub fn get_stderr(&mut self) -> &mut ChildStderr {
        self.child.stderr.as_mut().unwrap()
    }

    pub fn close_stdin(&mut self) -> Result<()> {
        self.child.stdin.take();
        Ok(())
    }

    pub fn command_string(&self) -> String {
        format!("{}", self.command)
    }

    pub async fn write_stdin(&mut self, s: &str) -> Result<()> {
        let stdin = self.child.stdin.as_mut().unwrap();
        stdin.write_all(s.as_bytes()).await?;
        stdin.flush().await?;
        Ok(())
    }
}
