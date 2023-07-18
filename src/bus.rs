use mpsc::error::SendError;
use tokio::sync::mpsc;

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
        if let Some(mut tx) = self.inner.get_mut(&tag) {
            if tx.is_closed() {
                return Err(SendError((tag, value)));
            }

            return tx.send(value).await.map(|_| None);
        }

        Ok(Some(self.make_lane(tag, value).await))
    }

    /// Closes all lanes.
    #[inline]
    pub fn clear(&mut self) {
        self.inner.clear();
    }

    #[inline]
    async fn make_lane(&self, tag: T, value: V) -> LaneRx<T, V> {
        let (tx, rx) = mpsc::channel(self.lane_buf);

        let mut lane_tx = LaneTx::new(tx, tag.clone());

        let tx_slot = lane_tx
            .send(value)
            .await
            .map(|_| self.inner.insert_or_replace(tag.clone(), lane_tx))
            .unwrap();

        LaneRx::new(rx, tx_slot)
    }
}
