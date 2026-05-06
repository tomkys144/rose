#[derive(Debug, Clone)]
pub struct Message {
    pub msg: String,
    pub msg_type: MessageType,
}

#[derive(Debug, Clone, Default)]
pub enum MessageType {
    #[default]
    Info,
    Warning,
    Error,
}
