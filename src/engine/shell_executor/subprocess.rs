//! Async execution and communication with child processes
use std::collections::HashMap;

use anyhow::{Result, Context, anyhow};

use async_process::{Command as AsyncCommand, Child, Stdio, ChildStdout};
use futures::AsyncWriteExt;
use async_std::io::ReadExt;

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

/*
/// Makes a POSIX pipe with a non-blocking read end that will not be
/// inherited by child processes, and a duplicated write end.
fn output_pipe() -> Result<(File, (File, File))> {
    use nix::fcntl::{fcntl, FcntlArg, OFlag};
    use nix::unistd::pipe;
    use nix::unistd::dup;
    use std::os::unix::io::{AsRawFd, FromRawFd};

    let (read, write) = pipe()?;
    fcntl(read.as_raw_fd(), FcntlArg::F_SETFL(OFlag::O_NONBLOCK | OFlag::O_CLOEXEC))?;

    let write_fds = (dup(write.as_raw_fd())?, write.as_raw_fd());
    let write_files = unsafe { (File::from_raw_fd(write_fds.0), File::from_raw_fd(write_fds.1)) };

    Ok((File::from(read), write_files))
}
*/
