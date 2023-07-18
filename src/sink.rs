use hashbrown::hash_map::DefaultHashBuilder;
use hashbrown::HashMap;
use tokio::sync::{mpsc, Mutex, MutexGuard};

use crate::stream::MuxStream;

/// A type alias for the [`HashMap`] used to store the [`Sender`]s for each
/// stream.
pub type SinkMap<T, V> = HashMap<T, mpsc::Sender<V>, DefaultHashBuilder>;

/// A multiplexed streams generator.
///
/// This struct is used to generate new [`MuxStream`]s by using the tags
/// received when [`MuxSink::push`] method is called.
#[derive(Debug)]
pub struct MuxSink<T, V> {
    streams_tx: Mutex<SinkMap<T, V>>,
    pending_tx: mpsc::Sender<MuxStream<T, V>>,
    mux_tx: mpsc::Sender<(T, V)>,
    stream_buf: usize,
}

impl<T, V> MuxSink<T, V>
where
    T: std::hash::Hash + Eq + Clone,
{
    /// Creates a new [`MuxSink`] with the given backlog and per-channel buffer
    /// size.
    ///
    /// # Parameters
    /// * `stream_buf` - The number of messages to buffer per stream.
    /// * `pending_tx` - The sender into which to send new streams.
    /// * `mux_tx` - The sender that channels use to send messages to the mux.
    #[inline]
    pub fn new(
        stream_buf: usize,
        pending_tx: mpsc::Sender<MuxStream<T, V>>,
        mux_tx: mpsc::Sender<(T, V)>,
    ) -> Self {
        assert!(stream_buf > 0);

        Self {
            streams_tx: Default::default(),
            pending_tx,
            mux_tx,
            stream_buf,
        }
    }

    /// Sends an item to the steam with the given tag if it exists; otherwise,
    /// creates a new stream and sends the item to it.
    ///
    /// # Parameters
    /// * `tag` - The tag of the channel to push to.
    /// * `value` - The item to push.
    ///
    /// # Returns
    /// A mutable reference to the sender of the channel.
    pub async fn send_to(
        &self,
        tag: T,
        value: V,
    ) -> Result<(), mpsc::error::SendError<(T, V)>> {
        let mut streams_tx = self.streams_tx.lock().await;

        if let Some(tx) = streams_tx.get_mut(&tag) {
            if tx.is_closed() {
                self.add_or_replace_stream(tag, value, streams_tx).await
            } else if let Err(err) = tx.send(value).await {
                self.add_or_replace_stream(tag, err.0, streams_tx).await
            } else {
                Ok(())
            }
        } else {
            self.add_or_replace_stream(tag, value, streams_tx).await
        }
    }

    /// Creates a new [`MuxSinkEntry`] with the given tag.
    ///
    /// The returned [`MuxSinkEntry`] can be used to send items to the stream
    /// with a fluent API.
    #[inline]
    pub fn stream(&mut self, tag: T) -> MuxSinkEntry<'_, T, V> {
        MuxSinkEntry { sink: self, tag }
    }

    /// Removes the stream with the given tag.
    #[inline]
    pub async fn remove(&self, tag: &T) {
        self.streams_tx.lock().await.remove(tag);
    }

    #[inline]
    async fn add_or_replace_stream(
        &self,
        tag: T,
        value: V,
        mut streams_tx: MutexGuard<'_, SinkMap<T, V>>,
    ) -> Result<(), mpsc::error::SendError<(T, V)>> {
        let (tx, rx) = mpsc::channel::<V>(self.stream_buf);
        let stream = MuxStream::new(tag.clone(), self.mux_tx.clone(), rx);

        if (self.pending_tx.send(stream).await).is_err() {
            return Err(mpsc::error::SendError((tag, value)));
        }

        tx.send(value).await.unwrap();

        if let Some(stream_tx) = streams_tx.get_mut(&tag) {
            *stream_tx = tx;
        } else {
            streams_tx.insert_unique_unchecked(tag, tx);
        }

        Ok(())
    }
}

/// A wrapper around a [`MuxSink`] that allows sending items to a stream with a
/// fluent API.
#[derive(Debug)]
pub struct MuxSinkEntry<'a, T, V> {
    sink: &'a MuxSink<T, V>,
    tag: T,
}

impl<'a, T, V> MuxSinkEntry<'a, T, V>
where
    T: Clone + Eq + std::hash::Hash,
{
    /// Sends an item to the stream with the given tag.
    #[inline]
    pub async fn send(
        self,
        value: V,
    ) -> Result<MuxSinkEntry<'a, T, V>, mpsc::error::SendError<(T, V)>> {
        self.sink.send_to(self.tag.clone(), value).await?;

        Ok(self)
    }

    // Removes the stream with the given tag.
    #[inline]
    pub async fn remove(self) {
        self.sink.remove(&self.tag).await
    }
}

impl<T, V> Drop for MuxSink<T, V> {
    fn drop(&mut self) {
        panic!("len: {}", self.streams_tx.get_mut().len());
    }
}
