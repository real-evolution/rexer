use hashbrown::hash_map::DefaultHashBuilder;
use hashbrown::HashMap;
use tokio::sync::mpsc;

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
    streams_tx: SinkMap<T, V>,
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
        &mut self,
        tag: T,
        value: V,
    ) -> Result<&mut mpsc::Sender<V>, mpsc::error::SendError<(T, V)>> {
        if let Some(chan) = self.streams_tx.get_mut(&tag) {
            if chan.is_closed() {
                self.add_or_replace_stream(tag, value).await
            } else if let Err(err) = chan.send(value).await {
                self.add_or_replace_stream(tag, err.0).await
            } else {
                Ok(self.streams_tx.get_mut(&tag).unwrap())
            }
        } else {
            self.add_or_replace_stream(tag, value).await
        }
    }

    }

    #[inline]
    async fn add_or_replace_stream(
        &mut self,
        tag: T,
        value: V,
    ) -> Result<&mut mpsc::Sender<V>, mpsc::error::SendError<(T, V)>> {
        let (tx, rx) = mpsc::channel::<V>(self.stream_buf);
        let stream = MuxStream::new(tag.clone(), self.mux_tx.clone(), rx);

        if (self.pending_tx.send(stream).await).is_err() {
            return Err(mpsc::error::SendError((tag, value)));
        }

        tx.send(value).await.unwrap();

        Ok(self
            .streams_tx
            .raw_entry_mut()
            .from_key(&tag)
            .and_modify(|_, o| *o = tx.clone())
            .or_insert_with(|| (tag, tx))
            .1)
    }
}

