use super::emit::Message;

/// Denotes data read from Stdout of a unit.  Can be either a text line, or an emit message
/// which Sysunit must handle
#[derive(Debug, PartialEq, Clone)]
pub enum StdoutData {
    TextLine(String),
    Message(Message),
}
