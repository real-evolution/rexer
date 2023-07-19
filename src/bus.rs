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
    pub async fn push(
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

        if tx.is_closed() {
            return Err(SendError((tag, value)));
        } else {
            tx.send(value).await?;
        }

        Ok(lane_rx)
    }

    /// Closes all lanes.
    #[inline]
    pub fn clear(&mut self) {
        self.inner.clear();
    }
}

impl<T: Key, V> Drop for Bus<T, V> {
    #[inline]
    fn drop(&mut self) {
        self.clear()
    }
}
