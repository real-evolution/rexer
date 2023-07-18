use tokio::sync::mpsc::error::SendError;
use tokio::sync::mpsc::{Receiver, Sender};

use crate::map::{Key, MapSlot};

/// A single tagged lane in [`Bus`](crate::bus::Bus).
#[derive(Debug)]
pub struct Lane<T: Key, V> {
    tx: LaneTx<T, V>,
    rx: LaneRx<T, V>,
}

impl<T: Key, V> Lane<T, V> {
    /// Create a new lane from a sender and receiver.
    ///
    /// # Panics
    /// Panics if the sender and receiver tags do not match.
    #[inline]
    pub(crate) fn from_parts(tx: LaneTx<T, V>, rx: LaneRx<T, V>) -> Self {
        assert_eq!(tx.tag(), rx.tag());

        Self { tx, rx }
    }

    /// Gets a reference to the lane's sender.
    #[inline]
    pub fn sender(&mut self) -> &mut LaneTx<T, V> {
        &mut self.tx
    }

    /// Gets a reference to the lane's receiver.
    #[inline]
    pub fn receiver(&mut self) -> &mut LaneRx<T, V> {
        &mut self.rx
    }

    /// Splits the lane into its sender and receiver.
    #[inline]
    pub fn split(self) -> (LaneTx<T, V>, LaneRx<T, V>) {
        (self.tx, self.rx)
    }
}

/// A [`Lane`](crate::lane::Lane) sender half.
#[derive(Debug, Clone)]
pub struct LaneTx<T: Key, V> {
    tag: T,
    inner: Sender<(T, V)>,
}

impl<T: Key, V> LaneTx<T, V> {
    /// Create a new lane sender.
    ///
    /// # Parameters
    /// * `inner` - The underlying channel sender.
    /// * `tag` - The tag of the lane.
    #[inline]
    pub(crate) const fn new(inner: Sender<(T, V)>, tag: T) -> Self {
        Self { tag, inner }
    }

    /// Sends a tagged value through the lane.
    ///
    /// # Parameters
    /// * `value` - The value to send.
    #[inline]
    pub async fn send(&mut self, value: V) -> Result<(), SendError<(T, V)>> {
        self.inner.send((self.tag.clone(), value)).await
    }

    /// Gets whether the lane is closed or not.
    #[inline]
    pub fn is_closed(&self) -> bool {
        self.inner.is_closed()
    }

    /// Gets the tag of the lane.
    #[inline]
    pub const fn tag(&self) -> &T {
        &self.tag
    }
}

/// A [`Lane`](crate::lane::Lane) receiver half.
#[derive(Debug)]
pub struct LaneRx<T: Key, V> {
    inner: Receiver<(T, V)>,
    tx_slot: MapSlot<T, LaneTx<T, V>>,
}

impl<T: Key, V> LaneRx<T, V> {
    /// Create a new lane receiver.
    ///
    /// # Parameters
    /// * `inner` - The underlying channel receiver.
    /// * `tx_slot` - The slot in the map for the lane's sender.
    #[inline]
    pub(crate) const fn new(
        inner: Receiver<(T, V)>,
        tx_slot: MapSlot<T, LaneTx<T, V>>,
    ) -> Self {
        Self { inner, tx_slot }
    }

    /// Receives a tagged value from the lane.
    #[inline]
    pub async fn recv(&mut self) -> Option<V> {
        match self.inner.recv().await {
            | Some((tag, value)) => {
                debug_assert_eq!(self.tag(), &tag);
                Some(value)
            }
            | None => None,
        }
    }

    /// Closes the lane, preventing new values from being sent.
    ///
    /// Any values already sent can still be received.
    #[inline]
    pub fn close(&mut self) {
        self.inner.close()
    }

    /// Gets whether the lane is closed or not.
    #[inline]
    pub const fn tag(&self) -> &T {
        self.tx_slot.key()
    }
}
