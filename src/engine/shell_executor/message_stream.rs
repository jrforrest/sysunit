use anyhow::{Result, anyhow, Context};

use futures::io::AsyncRead;

use crate::models::{EmitMessage, OpStatus, ValueSet, Meta, CheckPresence, StdoutData, Dependency};
use crate::parser::{parse_deps, parse_params, parse_value};
use crate::events::{OpEventHandler, OpEvent};

use super::stdout_data::StdoutDataProducer;

/// Provides a useful interface for reading messages from the emit channel
pub struct MessageStream<R: AsyncRead + Unpin> {
    data_producer: StdoutDataProducer<R>,
}

impl<R: AsyncRead + Unpin> MessageStream<R> {
    pub fn new(data_producer: StdoutDataProducer<R>) -> Self {
        Self { data_producer }
    }

    /// When a unit script command is run, it will emit messages during the operation,
    /// and will end with a status message.
    ///
    /// This function reads messages from the emit pipe until it encounters a status
    /// message, at which point it returns the status and the messages that were read.
    async fn drain_messages(&mut self, ev_handler: OpEventHandler) -> Result<(OpStatus, Vec<EmitMessage>)> {
        let mut messages = Vec::new();

        loop {
            let message = self.data_producer.next().await?;
            match message {
                None => return Err(anyhow!("Unexpected EOF parsing output stream")),
                Some(data) => {
                    ev_handler.handle(OpEvent::Output(data.clone()))?;
                    match data {
                        StdoutData::TextLine(_) => (),
                        StdoutData::Message(emit_message) => {
                            if emit_message.header.name == "status" {
                                let status = OpStatus::from_code(&emit_message.text)?;
                                return Ok((status, messages));
                            } else {
                                messages.push(emit_message);
                            }
                        }
                    }
                }
            }
        }
    }

    /// For most operations, we expect a stream of messages of a single type, followed by a status.
    ///
    /// This function will read messages from the emit pipe until it encounters a status message,
    /// and will return the status and the messages of the given type that were read.
    ///
    /// It will return an error if a message of a different type is encountered.
    pub async fn drain_messages_of_type(&mut self, message_type: &str, ev_handler: OpEventHandler) -> Result<(OpStatus, Vec<EmitMessage>)> {
        let mut messages = Vec::new();
        let (status, drained_messages) = self.drain_messages(ev_handler).await?;

        for message in drained_messages {
            if message.header.name == message_type {
                messages.push(message);
            } else {
                return Err(anyhow!("Unexpected message type: {:?}", message));
            }
        }

        Ok((status, messages))
    }

    /// Retreives and parses a series of dependency messages from the emit pipe,
    /// fails if the operation does not return a successful status.
    pub async fn get_deps(&mut self, ev_handler: OpEventHandler) -> Result<Vec<Dependency>> {
        let (status, messages) = self.drain_messages_of_type("dep", ev_handler).await?;
        status.expect_ok()?;
        let mut deps = Vec::new();
        for dep_msg in messages.iter() {
            let parsed_deps = parse_deps(&dep_msg.text)?;
            deps.extend(parsed_deps);
        }
        Ok(deps)
    }

    /// Retrieves parameter specification messages from the emit pipe,
    /// fails if the operation does not return a successful status.
    pub async fn get_meta(&mut self, ev_handler: OpEventHandler) -> Result<Meta> {
        let (status, messages) = self.drain_messages_of_type("meta", ev_handler).await?;
        status.expect_ok()?;

        let mut meta = Meta::empty();

        for message in messages.iter() {
            let field = match message.header.field {
                Some(ref f) => f,
                None => return Err(anyhow!("Meta message missing field")),
            };

            match field.as_str() {
                "author" => meta.author = Some(message.text.clone()),
                "desc" => meta.desc = Some(message.text.clone()),
                "version" => meta.version = Some(message.text.clone()),
                "params" => {
                    meta.params = parse_params(&message.text)?;
                },
                _ => return Err(anyhow!("Unexpected message type for meta operation: {:?}", message)),
            }
        }
        Ok(meta)
    }

    // Retrieves check values from the emit pipe
    //
    // The check operation can emit values, and also emits a presence message to indicate if the
    // unit is present on the system. This defaults to false.
    pub async fn get_check_values(&mut self, ev_handler: OpEventHandler) -> Result<(bool, ValueSet)> {
        let (status, messages) = self.drain_messages(ev_handler).await?;
        status.expect_ok()?;
        let mut vset = ValueSet::new();
        // Units are presumed to not be present by default
        let mut present: Option<CheckPresence> = None;

        for message in messages {
            match message.header.name.as_str() {
                "present" => {
                    if present.is_some() {
                        return Err(anyhow!("Multiple presence messages in check operation"));
                    }

                    let presence_bool = message.text.parse()
                        .context(format!("Could not parse present message body, expected 'true' or 'false, got: {}", message.text))?;
                    present = Some(presence_bool)
                },
                "value" => {
                    let value = parse_value(&message.text)
                        .context("could not parse emitted value")?;
                    let key = match message.header.field {
                        Some(key) => key.clone(),
                        None => return Err(anyhow!("Value message missing field")),
                    };
                    vset.add_value(&key, value);
                },
                _ => return Err(anyhow!("Unexpected message type for check operation: {:?}", message)),
            }
        }

        let check_presence = present.unwrap_or_default();

        Ok((check_presence, vset))
    }

    pub async fn get_apply_values(&mut self, ev_handler: OpEventHandler) -> Result<ValueSet> {
        let (status, vset) = self.get_values(ev_handler).await?;
        status.expect_ok()?;
        Ok(vset)
    }

    pub async fn get_remove_values(&mut self, ev_handler: OpEventHandler) -> Result<ValueSet> {
        let (status, vset) = self.get_values(ev_handler).await?;
        status.expect_ok()?;
        Ok(vset)
    }

    async fn get_values(&mut self, ev_handler: OpEventHandler) -> Result<(OpStatus, ValueSet)> {
        let mut vset = ValueSet::new();
        let (status, drained_messages) = self.drain_messages_of_type("value", ev_handler).await?;

        for message in drained_messages {
            let value = parse_value(&message.text)?;
            let key = match message.header.field {
                Some(key) => key.clone(),
                None => return Err(anyhow!("Value message missing field")),
            };
            vset.add_value(&key, value);
        }

        Ok((status, vset))
    }

    pub fn finalize(self) -> Result<()> {
        self.data_producer.finalize()
    }
}
