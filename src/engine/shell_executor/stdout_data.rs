use tracing::instrument;
use anyhow::{Result, anyhow};
use futures::io::AsyncRead;
use async_std::io::ReadExt;

use crate::models::StdoutData;
use crate::parser::{StreamingResult, parse_stdout_data};

/// Parses output from a unit into text lines and messages
pub struct StdoutDataProducer<R: AsyncRead + Unpin> {
    read_fd: R,
    buf: String,
}

impl<R: AsyncRead + Unpin> StdoutDataProducer<R> {
    pub fn new(read_fd: R) -> Self {
        Self { read_fd, buf: String::new() }
    }
    // Reads the next bit of data from stdout FD
    #[instrument(skip(self))]
    pub async fn next(&mut self) -> Result<Option<StdoutData>> {
        use StreamingResult::*;

        let buf_len = self.buf.len();
        // first, fill the buf it is empty
        if buf_len == 0 && !self.read_more().await? {
            // buf is empty, pipe is closed, and we're not in the middle of parsing a
            // message so we can exit normally.
            return Ok(None);
        }

        loop {
            // try to parse what's in the buffer
            match parse_stdout_data(&self.buf) {
                Complete(Ok((remaining, message))) => {
                    self.buf = remaining.to_string();
                    return Ok(Some(message));
                },
                Complete(Err(e)) => return Err(e.context(format!("Error parsing emit message: {}", self.buf))),
                Incomplete => {
                    if !self.read_more().await? {
                        // If we can't read more data, but we're in the middle of parsing a message, we
                        // have an unexpected EOF.
                        return Err(anyhow!("Unexpected EOF parsing emit stream! buffer: {:?}", self.buf));
                    } else {
                        continue;
                    }
                },
            };
        }
    }

    /// Reads more data into the buffer
    async fn read_more(&mut self) -> Result<bool> {
        let mut tmp_buf = [0u8; 1024];

        let n_bytes_read = self.read_fd.read(&mut tmp_buf).await?;

        if n_bytes_read == 0 {
            return Ok(false)
        }

        let as_str = String::from_utf8_lossy(&tmp_buf[..n_bytes_read]);

        self.buf.push_str(&as_str);
        Ok(true)
    }

    pub fn finalize(self) -> Result<()> {
        drop(self); // Implicitly closes the read fd
        Ok(())
    }
}
