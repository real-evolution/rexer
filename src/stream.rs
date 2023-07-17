use tokio::sync::mpsc;

use crate::channel::tagged::{TaggedReceiver, TaggedSender};

/// A multiplexed stream.
#[derive(Debug)]
pub struct MuxStream<T, V> {
    tx: TaggedSender<T, V>,
    rx: TaggedReceiver<T, V>,
}

impl<T, V> MuxStream<T, V>
where
    T: Clone,
{
    /// Create a new [`MuxStream`] with the given tag, sender, and receiver.
    ///
    /// # Parameters
    /// * `tag` - The tag of the stream.
    /// * `sender` - A copy of the master mux sender.
    /// * `receiver` - The receiver for the stream.
    #[inline]
    pub fn new(
        tag: T,
        sender: mpsc::Sender<(T, V)>,
        receiver: mpsc::Receiver<V>,
    ) -> Self {
        Self {
            tx: TaggedSender::new(tag.clone(), sender),
            rx: TaggedReceiver::new(tag, receiver),
        }
    }

    /// Split the stream into its sender and receiver, consuming [`self`].
    #[inline]
    pub fn split(self) -> (TaggedSender<T, V>, TaggedReceiver<T, V>) {
        (self.tx, self.rx)
    }
}
