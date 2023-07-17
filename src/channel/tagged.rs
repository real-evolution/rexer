use tokio::sync::mpsc::error::SendError;
use tokio::sync::mpsc::{Receiver, Sender};

/// A tagged channel sender.
///
/// Every message sent through this sender will be tagged with the tag given
/// when the channel was created.
#[derive(Debug, Clone)]
pub struct TaggedSender<T, V> {
    inner: Sender<(T, V)>,
    tag: T,
}

impl<T, V> TaggedSender<T, V>
where
    T: Clone,
{
    /// Create a new [`TaggedSender`] with the given tag and sender.
    ///
    /// # Parameters
    /// * `tag` - The tag to be used to tag messages sent through the sender.
    /// * `inner` - The sender to use to send tagged messages through.
    #[inline]
    pub const fn new(tag: T, inner: Sender<(T, V)>) -> Self {
        Self { inner, tag }
    }

    /// Send a message through the channel.
    ///
    /// This will tag the message with the tag given when the channel was
    /// created, so only the value is needed.
    ///
    /// # Parameters
    /// * `value` - The message to send.
    #[inline]
    pub async fn send(&mut self, value: V) -> Result<(), SendError<(T, V)>> {
        self.inner.send((self.tag.clone(), value)).await
    }
}

/// A tagged channel receiver.
///
/// Every message received from this receiver will be tagged with the tag
/// given when the channel was created.
#[derive(Debug)]
pub struct TaggedReceiver<T, V> {
    inner: Receiver<V>,
    tag: T,
}

impl<T, V> TaggedReceiver<T, V>
where
    T: Clone,
{
    /// Create a new [`TaggedReceiver`] with the given tag and receiver.
    #[inline]
    pub const fn new(tag: T, inner: Receiver<V>) -> Self {
        Self { inner, tag }
    }

    /// Receive a message from the channel.
    #[inline]
    pub async fn recv(&mut self) -> Option<(T, V)> {
        self.inner.recv().await.map(|v| (self.tag.clone(), v))
    }
}
