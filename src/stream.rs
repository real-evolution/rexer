use std::hash::Hash;
use std::sync::Arc;

use tokio::sync::mpsc;

use crate::channel::tagged::{TaggedReceiver, TaggedSender};
use crate::sink::SinkMap;

/// A multiplexed stream.
#[derive(Debug)]
pub struct MuxStream<T, V>
where
    T: Clone + Eq + std::hash::Hash,
{
    tx: TaggedSender<T, V>,
    rx: TaggedReceiver<T, V>,
    senders_map: Arc<SinkMap<T, V>>,
}

impl<T, V> MuxStream<T, V>
where
    T: Clone + Eq + Hash,
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
        senders_map: Arc<SinkMap<T, V>>,
    ) -> Self {
        Self {
            tx: TaggedSender::new(tag.clone(), sender),
            rx: TaggedReceiver::new(tag, receiver),
            senders_map,
        }
    }

    /// Split the stream into its sender and receiver, consuming [`self`].
    #[inline]
    pub fn parts(
        &mut self,
    ) -> (&mut TaggedSender<T, V>, &mut TaggedReceiver<T, V>) {
        (&mut self.tx, &mut self.rx)
    }
}

impl<T, V> Drop for MuxStream<T, V>
where
    T: Clone + Eq + Hash,
{
    #[inline]
    fn drop(&mut self) {
        self.senders_map.remove(self.tx.tag());
    }
}
