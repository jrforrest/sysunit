//! Async execution and communication with child processes
use std::collections::HashMap;

use anyhow::{Result, Context};

use async_process::{Command as AsyncCommand, Child, Stdio, ChildStdout};
use futures::AsyncWriteExt;

pub struct Command {
    pub cmd: String,
    pub args: Vec<String>,
    pub env: HashMap<String, String>
}

#[derive(Debug)]
pub struct Subprocess {
    child: Child,
}

impl Subprocess {
    pub fn init(cmd: Command) -> Result<Self> {
        let child = AsyncCommand::new(&cmd.cmd)
            .args(&cmd.args)
            .envs(&cmd.env)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .context("Could not spawn subprocess")?;

        Ok(Self {
            child,
        })
    }

    /// Close all IO and wait for the subprocess to exit
    pub async fn finalize(mut self) -> Result<i32> {
        let status = self.child.status().await?;
        Ok(status.code().unwrap())
    }

    pub fn get_stdout(&mut self) -> &mut ChildStdout {
        self.child.stdout.as_mut().unwrap()
    }

    pub async fn write_stdin(&mut self, s: &str) -> Result<()> {
        let stdin = self.child.stdin.as_mut().unwrap();
        stdin.write_all(s.as_bytes()).await?;
        stdin.flush().await?;
        Ok(())
    }
}
