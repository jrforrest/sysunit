use crate::events::OpEventHandler;
use crate::models::{Operation, OpResult, EmitMessage, OpStatus};
use async_std::fs::File;

use anyhow::Result;

pub struct OpExecutor<'a> {
    ev_handler: OpEventHandler,
    op: Operation,
    input: &'a mut File,
}

/// Executes the given operation on the given subprocess, reporting events via the ev_handler
impl <'a> OpExecutor <'a> {
    pub fn new(op: Operation, ev_handler: OpEventHandler, input: &'a mut File) -> Self {
        Self { op, ev_handler, input }
    }

    /// When a unit script command is run, it will emit messages during the operation,
    /// and will end with a status message.
    ///
    /// This function reads messages from the emit pipe until it encounters a status
    /// message, at which point it returns the status and the messages that were read.
    async fn drain_messages(&self) -> Result<(OpStatus, Vec<EmitMessage>)> {
        let mut messages = Vec::new();
        loop {
            let message = self.message_rx.recv().await?;
            if message.header.name == "status" {
                let status = OpStatus::from_code(&message.text)?;
                return Ok((status, messages));
            } else {
                messages.push(message);
            }
        }
    }

    /// For most operations, we expect a stream of messages of a single type, followed by a status.
    ///
    /// This function will read messages from the emit pipe until it encounters a status message,
    /// and will return the status and the messages of the given type that were read.
    ///
    /// It will return an error if a message of a different type is encountered.
    pub async fn drain_messages_of_type(&self, message_type: &str)
        -> Result<(OpStatus, Vec<EmitMessage>)>
    {
        let mut messages = Vec::new();
        let (status, drained_messages) = self.drain_messages().await?;

        for message in drained_messages {
            if message.header.name == message_type {
                messages.push(message);
            } else {
                return Err(anyhow!("Unexpected message type: {:?}", message));
            }
        }

        Ok((status, messages))
    }

    pub async fn result() -> Result<OpResult> {
    }
}

