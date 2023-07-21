use std::pin::Pin;
use std::task::{Context, Poll};

use futures::Sink;
use tokio_util::sync::{PollSendError, PollSender};

use crate::{Key, LaneTx};

pin_project_lite::pin_project! {
    /// A "poll-able" variant of [`LaneTx<T, V>`]. This is useful for when you
    /// want a type that implements [`Sink<(T, V)>`], as the current
    /// [`LaneTx<T, V>`] *does not* currently support polling.
    #[derive(Debug)]
    pub struct LaneSink<T, V> {
        #[pin]
        inner: PollSender<(T, V)>,
        tag: T,
    }
}

impl<T, V> LaneSink<T, V>
where
    T: Key + Send + 'static,
    V: Send + 'static,
{
    /// Constructs a new [`LaneTx<T, V>`] instance from the given sender.
    #[inline]
    pub fn new(sender: LaneTx<T, V>) -> Self {
        let (tag, sender) = sender.into_inner();

        Self {
            tag,
            inner: PollSender::new(sender),
        }
    }

    /// Gets a reference to the inner lane sender's tag.
    #[inline]
    pub const fn tag(&self) -> &T {
        &self.tag
    }

    /// Gets whether the inner lane sender is closed or not.
    #[inline]
    pub fn is_closed(&self) -> bool {
        self.inner.is_closed()
    }

    /// Closes the inner lane receiver.
    #[inline]
    pub fn close(&mut self) {
        self.inner.close();
    }
}

impl<T, V> Sink<V> for LaneSink<T, V>
where
    T: Key + Send + 'static,
    V: Send + 'static,
{
    type Error = PollSendError<(T, V)>;

    #[inline(always)]
    fn poll_ready(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Result<(), Self::Error>> {
        self.project().inner.poll_ready(cx)
    }

    #[inline(always)]
    fn start_send(self: Pin<&mut Self>, item: V) -> Result<(), Self::Error> {
        let tag = self.tag.clone();

        self.project().inner.start_send((tag, item))
    }

    #[inline(always)]
    fn poll_flush(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Result<(), Self::Error>> {
        self.project().inner.poll_flush(cx)
    }

    #[inline(always)]
    fn poll_close(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Result<(), Self::Error>> {
        self.project().inner.poll_close(cx)
    }
}

impl<T, V> From<LaneTx<T, V>> for LaneSink<T, V>
where
    T: Key + Send + 'static,
    V: Send + 'static,
{
    #[inline(always)]
    fn from(sender: LaneTx<T, V>) -> Self {
        Self::new(sender)
    }
}

impl<T, V> LaneTx<T, V>
where
    T: Key + Send + 'static,
    V: Send + 'static,
{
    /// Converts this lane sender into a [`LaneSink<T, V>`].
    ///
    /// This is only available when the `util` feature is enabled.
    #[inline]
    pub fn into_sink(self) -> LaneSink<T, V> {
        self.into()
    }
}
