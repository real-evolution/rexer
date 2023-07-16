use thiserror::Error;

/// Error type for channel receive operation errors.
#[derive(Debug, Error)]
pub enum ConsumeError {
    /// Indicates that the channel is closed on both sides with a call to
    /// `close()`
    #[error("consumer is closed")]
    Closed,

    /// Indicates that all sender instances are dropped and the channel is
    /// closed from the send side.
    #[error("producer is closed")]
    ProducerClosed,

    /// Indicates that channel operation reached timeout and is canceled.
    #[error("channel operation timeout reached")]
    Timeout,
}

#[derive(Debug, Error)]
pub enum ProduceError {
    /// Indicates that the channel is closed on both sides with a call to
    /// `close()`.
    #[error("channel is closed")]
    Closed,

    /// Indicates that all receiver instances are dropped and the channel is
    /// closed from the receive side.
    #[error("consumer is closed")]
    ConsumerClosed,
}

impl From<kanal::OneshotReceiveError> for ConsumeError {
    fn from(_: kanal::OneshotReceiveError) -> Self {
        Self::Closed
    }
}

impl From<kanal::ReceiveError> for ConsumeError {
    fn from(value: kanal::ReceiveError) -> Self {
        match value {
            | kanal::ReceiveError::Closed => Self::Closed,
            | kanal::ReceiveError::SendClosed => Self::ProducerClosed,
        }
    }
}

impl From<kanal::SendError> for ProduceError {
    fn from(value: kanal::SendError) -> Self {
        match value {
            | kanal::SendError::Closed => Self::Closed,
            | kanal::SendError::ReceiveClosed => Self::ConsumerClosed,
        }
    }
}

impl From<kanal::SendErrorTimeout> for ProduceError {
    fn from(value: kanal::SendErrorTimeout) -> Self {
        match value {
            | kanal::SendErrorTimeout::Closed => Self::Closed,
            | kanal::SendErrorTimeout::ReceiveClosed => Self::ConsumerClosed,
            | kanal::SendErrorTimeout::Timeout => Self::ConsumerClosed,
        }
    }
}
