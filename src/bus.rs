use tokio::sync::mpsc;
use tokio::sync::mpsc::error::SendError;

use crate::lane::{LaneRx, LaneTx};
use crate::map::{Key, Map};

/// The backing bus for [`Mux`](crate::mux::Mux).
///
/// This acts as a single point of entry for all lanes, and is responsible for
/// distributing messages to the appropriate lanes.
#[derive(Debug)]
pub struct Bus<T: Key, V> {
    inner: Map<T, LaneTx<T, V>>,
    lane_buf: usize,
}

impl<T: Key, V> Bus<T, V> {
    /// Create a new instance of [`Bus`].
    ///
    /// # Parameters
    /// * `lane_buf` - The buffer size for each lane.
    #[inline]
    pub fn new(lane_buf: usize) -> Self {
        Self {
            inner: Default::default(),
            lane_buf,
        }
    }

    /// Gets a reference to the lane with the given tag.
    ///
    /// A new lane will be created if one does not already exist.
    ///
    /// # Parameters
    /// * `tag` - The tag of the lane.
    /// * `value` - The value to send to the lane.
    pub async fn push(&self, mut tag: T, mut value: V) -> Option<LaneRx<T, V>> {
        loop {
            match self.push_item(tag, value).await {
                | Ok(rx) => return rx,
                | Err(SendError((etag, evalue))) => {
                    (tag, value) = (etag, evalue);
                }
            }
        }
    }

    /// Closes all lanes.
    #[inline]
    pub fn clear(&mut self) {
        self.inner.clear();
    }

    #[inline]
    async fn push_item(
        &self,
        tag: T,
        value: V,
    ) -> Result<Option<LaneRx<T, V>>, SendError<(T, V)>> {
        let mut lane_rx = None;
        let mut tx = self.inner.get_or_insert(tag.clone(), |slot| {
            let (tx, rx) = mpsc::channel(self.lane_buf);

            lane_rx = Some(LaneRx::new(rx, slot));

            LaneTx::new(tx, tag.clone())
        });

        match lane_rx {
            | Some(lane_rx) => {
                tx.send(value).await.unwrap();
                Ok(Some(lane_rx))
            }
            | None => {
                if tx.is_closed() {
                    // remove closed lane from map
                    _ = self.inner.remove(&tag);

                    return Err(SendError((tag, value)));
                }

                tx.send(value).await.map(|_| None)
            }
        }
    }
}

impl<T: Key, V> Drop for Bus<T, V> {
    #[inline]
    fn drop(&mut self) {
        self.clear()
    }
}
