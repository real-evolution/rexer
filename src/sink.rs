use std::collections::HashMap;

use tokio::sync::mpsc::error::SendError;
use tokio::sync::mpsc::{self, Sender};

use crate::stream::MuxStream;

/// A multiplexed streams generator.
///
/// This struct is used to generate new [`MuxStream`]s by using the tags
/// received when [`MuxSink::push`] method is called.
#[derive(Debug)]
pub struct MuxSink<T, V> {
    chans: HashMap<T, Sender<V>>,
    streams_tx: Sender<MuxStream<T, V>>,
    mux_tx: Sender<(T, V)>,
    chan_buf: usize,
}

impl<T, V> MuxSink<T, V>
where
    T: std::hash::Hash + Eq + Clone,
{
    /// Creates a new [`MuxSink`] with the given backlog and per-channel buffer
    /// size.
    ///
    /// # Parameters
    /// * `chan_buf` - The number of messages to buffer per channel.
    /// * `streams_tx` - The sender into which to send new streams.
    /// * `mux_tx` - The sender that channels use to send messages to the mux.
    #[inline]
    pub fn new(
        chan_buf: usize,
        acceptor_tx: Sender<MuxStream<T, V>>,
        mux_tx: Sender<(T, V)>,
    ) -> Self {
        assert!(chan_buf > 0);

        Self {
            chans: HashMap::new(),
            streams_tx: acceptor_tx,
            mux_tx,
            chan_buf,
        }
    }

    /// Pushes an item to the sink with the given tag.
    ///
    /// If a stream with the specified tag exists, the item will be sent to it;
    /// otherwise, it will be created, and the stream will be buffered until
    /// it is accepted.
    ///
    /// # Parameters
    /// * `tag` - The tag of the channel to push to.
    /// * `value` - The item to push.
    pub async fn push(
        &mut self,
        tag: T,
        value: V,
    ) -> Result<(), SendError<(T, V)>> {
        if let Some(chan) = self.chans.get_mut(&tag) {
            if let Err(err) = chan.send(value).await {
                self.insert_new(tag, err.0).await?;
            }
        } else {
            self.insert_new(tag, value).await?;
        }

        Ok(())
    }

    #[inline]
    async fn insert_new(
        &mut self,
        tag: T,
        value: V,
    ) -> Result<(), SendError<(T, V)>> {
        println!("new, {}", self.chans.len());

        let (tx, rx) = mpsc::channel::<V>(self.chan_buf);
        let stream = MuxStream::new(tag.clone(), self.mux_tx.clone(), rx);

        if (self.streams_tx.send(stream).await).is_err() {
            return Err(SendError((tag, value)));
        }

        tx.send(value).await.unwrap();
        self.chans.insert(tag, tx);

        Ok(())
    }
}
