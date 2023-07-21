use std::task::{Context, Poll};

use tokio::sync::mpsc::error::SendError;
use tokio::sync::mpsc::{Receiver, Sender};

use crate::map::{Key, MapSlot};

pub(crate) type LaneTxSlot<T, V> = MapSlot<T, LaneTx<T, V>>;

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

pin_project_lite::pin_project! {
    /// A [`Lane`](crate::lane::Lane) sender half.
    #[derive(Debug, Clone)]
    pub struct LaneTx<T: Key, V> {
        #[pin]
        inner: Sender<(T, V)>,
        tag: T,
    }
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
    tx_slot: LaneTxSlot<T, V>,
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
        if self.is_closed() {
            return None;
        }

        let value = self.inner.recv().await;

        self.map_value(value)
    }

    /// Polls to receive the next message on this channel.
    #[inline]
    pub fn poll_recv(&mut self, cx: &mut Context<'_>) -> Poll<Option<V>>
    where
        T: Unpin,
    {
        if self.is_closed() {
            return Poll::Ready(None);
        }

        self.inner.poll_recv(cx).map(|v| self.map_value(v))
    }

    /// Gets whether the lane is closed or not.
    #[inline]
    pub const fn is_closed(&self) -> bool {
        !self.tx_slot.is_valid()
    }

    /// Closes the lane, preventing new values from being sent.
    ///
    /// Any values already sent can still be received.
    #[inline]
    pub fn close(&mut self) {
        if !self.is_closed() {
            self.inner.close();
            self.tx_slot.manual_drop();
        }
    }

    /// Gets whether the lane is closed or not.
    #[inline]
    pub const fn tag(&self) -> &T {
        self.tx_slot.key()
    }

    #[inline]
    fn map_value(&mut self, value: Option<(T, V)>) -> Option<V> {
        match value {
            | Some((tag, value)) => {
                debug_assert_eq!(self.tag(), &tag);
                Some(value)
            }
            | None => {
                self.tx_slot.manual_drop();
                None
            }
        }
    }
}
