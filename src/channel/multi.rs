use std::pin::Pin;
use std::task::{Context, Poll};

use futures::stream::FusedStream;
use futures::{Stream, StreamExt};

/// A multi-producer, multi-consumer sender side.
#[derive(Debug, Clone)]
pub struct MultiProducer<T>(kanal::AsyncSender<T>);

impl<T> MultiProducer<T> {
    /// Sends a value to the consumer.
    ///
    /// # Parameters
    /// - `value`: The value to send.
    #[inline]
    pub async fn send(
        &self,
        value: T,
    ) -> Result<(), super::error::ProduceError> {
        self.0.send(value).await.map_err(Into::into)
    }

    /// Sends a value to the consumer synchronously.
    ///
    /// # Parameters
    /// - `value`: The value to send.
    #[inline]
    pub fn send_sync(
        &self,
        value: T,
    ) -> Result<(), super::error::ProduceError> {
        self.0.as_sync().send(value).map_err(Into::into)
    }

    /// Gets whether this producer is disconnected from its corrosponding
    /// consumer or not.
    #[inline]
    pub fn is_disconnected(&self) -> bool {
        self.0.is_disconnected()
    }

    /// Gets whether the channel is closed or not.
    #[inline]
    pub fn is_closed(&self) -> bool {
        self.0.is_closed()
    }
}

/// A multi-producer, multi-consumer consumer side.
#[derive(Debug, Clone)]
pub struct MultiConsumer<T>(kanal::AsyncReceiver<T>);

impl<T> MultiConsumer<T> {
    /// Receives a value from the producer.
    #[inline]
    pub async fn recv(&self) -> Result<T, super::error::ConsumeError> {
        self.0.recv().await.map_err(Into::into)
    }

    /// Returns a [`futures::Stream`] of values from the producer.
    #[inline]
    #[must_use]
    pub fn stream(&self) -> MultiConsumerStream<'_, T> {
        MultiConsumerStream(self.0.stream())
    }

    /// Gets whether this consumer is disconnected from its corrosponding
    /// producer or not.
    #[inline]
    pub fn is_disconnected(&self) -> bool {
        self.0.is_disconnected()
    }

    /// Gets whether the channel is closed or not.
    #[inline]
    pub fn is_closed(&self) -> bool {
        self.0.is_closed()
    }

    /// Gets whether the channel is terminated or not.
    #[inline]
    pub fn is_terminated(&self) -> bool {
        self.0.is_terminated()
    }
}

/// A [`futures::Stream`] wrapper of [`MultiConsumer<T>`]
#[derive(Debug)]
pub struct MultiConsumerStream<'a, T>(kanal::ReceiveStream<'a, T>);

impl<'a, T> Stream for MultiConsumerStream<'a, T> {
    type Item = T;

    #[inline]
    fn poll_next(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Option<Self::Item>> {
        self.0.poll_next_unpin(cx)
    }
}

impl<'a, T> FusedStream for MultiConsumerStream<'a, T> {
    #[inline]
    fn is_terminated(&self) -> bool {
        self.0.is_terminated()
    }
}

/// Creates an unbounded multi-producer, multi-consumer channel.
#[inline]
#[must_use]
pub fn unbounded<T>() -> (MultiProducer<T>, MultiConsumer<T>) {
    let (sender, receiver) = kanal::unbounded_async();

    (MultiProducer(sender), MultiConsumer(receiver))
}

/// Creates a bounded multi-producer, multi-consumer channel.
///
/// # Parameters
/// * `bound` - The maximum number of elements the channel can hold.
#[inline]
#[must_use]
pub fn bounded<T>(bound: usize) -> (MultiProducer<T>, MultiConsumer<T>) {
    let (sender, receiver) = kanal::bounded_async(bound);

    (MultiProducer(sender), MultiConsumer(receiver))
}

/// Creates an optionally bounded multi-producer, multi-consumer channel.
///
/// # Parameters
/// * `bound` - If [`Some(usize)`], this value is maximum number of elements the
///   channel can hold.
#[inline]
#[must_use]
pub fn maybe_bounded<T>(
    bound: Option<usize>,
) -> (MultiProducer<T>, MultiConsumer<T>) {
    bound.map(bounded).unwrap_or_else(unbounded)
}
