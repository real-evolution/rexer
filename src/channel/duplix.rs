use super::multi;

#[derive(Debug, Clone)]
pub struct DuplixSide<T, R> {
    tx: multi::MultiProducer<T>,
    rx: multi::MultiConsumer<R>,
}

impl<T, R> DuplixSide<T, R> {
    /// Sends a value to the consumer of the other side.
    ///
    /// # Parameters
    /// - `value`: The value to send.
    #[inline]
    pub async fn send(
        &self,
        value: T,
    ) -> Result<(), super::error::ProduceError> {
        self.tx.send(value).await
    }

    /// Receives a value from the producer.
    #[inline]
    pub async fn recv(&self) -> Result<R, super::error::ConsumeError> {
        self.rx.recv().await
    }

    /// Gets whether the channel is closed or not.
    #[inline]
    pub fn is_closed(&self) -> bool {
        self.tx.is_closed() || self.rx.is_closed()
    }

    /// Decomposes this into its producer and consumer.
    #[inline]
    pub fn split(self) -> (multi::MultiProducer<T>, multi::MultiConsumer<R>) {
        (self.tx, self.rx)
    }

    #[inline]
    fn new(tx: multi::MultiProducer<T>, rx: multi::MultiConsumer<R>) -> Self {
        Self { tx, rx }
    }
}

/// Creates an unbounded (on both sides) duplix channel.
#[inline]
#[must_use]
pub fn unbounded<T, R>() -> (DuplixSide<T, R>, DuplixSide<R, T>) {
    let (tx1, rx2) = multi::unbounded::<T>();
    let (tx2, rx1) = multi::unbounded::<R>();

    (DuplixSide::new(tx1, rx1), DuplixSide::new(tx2, rx2))
}

/// Creates a bounded (on both sides) duplix channel.
///
/// # Parameters
/// * `bound` - The maximum number of elements the channel can hold.
#[inline]
#[must_use]
pub fn bounded<T, R>(bound: usize) -> (DuplixSide<T, R>, DuplixSide<R, T>) {
    let (tx1, rx2) = multi::bounded::<T>(bound);
    let (tx2, rx1) = multi::bounded::<R>(bound);

    (DuplixSide::new(tx1, rx1), DuplixSide::new(tx2, rx2))
}

/// Creates an optionally bounded multi-producer, multi-consumer channel.
///
/// # Parameters
/// * `bound` - If [`Some(usize)`], this value is maximum number of elements the
///   channel can hold.
#[inline]
#[must_use]
pub fn maybe_bounded<T, R>(
    bound: Option<usize>,
) -> (DuplixSide<T, R>, DuplixSide<R, T>) {
    bound.map(bounded).unwrap_or_else(unbounded)
}
