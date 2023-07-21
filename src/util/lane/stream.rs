use std::pin::Pin;
use std::task::{Context, Poll};

use futures::stream::FusedStream;
use futures::Stream;

use crate::{Key, LaneRx};

/// An adapter for [`LaneRx<T, V>`] that implements [`Stream`](futures::Stream)
/// and [`FusedStream`](futures::stream::FusedStream) traits.
#[derive(Debug)]
pub struct LaneStream<T: Key, V>(LaneRx<T, V>);

impl<T, V> LaneStream<T, V>
where
    T: Key + Unpin,
{
    /// Creates a new [`LaneStream`] from the given [`LaneRx<T, V>`].
    #[inline(always)]
    pub fn new(receiver: LaneRx<T, V>) -> Self {
        Self(receiver)
    }

    /// Deconstructs the [`LaneStream`] into its inner [`LaneRx<T, V>`].
    #[inline(always)]
    pub fn into_inner(self) -> LaneRx<T, V> {
        self.0
    }
}

impl<T: Key, V> AsRef<LaneRx<T, V>> for LaneStream<T, V> {
    #[inline(always)]
    fn as_ref(&self) -> &LaneRx<T, V> {
        &self.0
    }
}

impl<T: Key, V> AsMut<LaneRx<T, V>> for LaneStream<T, V> {
    #[inline(always)]
    fn as_mut(&mut self) -> &mut LaneRx<T, V> {
        &mut self.0
    }
}

impl<T, V> Stream for LaneStream<T, V>
where
    T: Key + Unpin,
{
    type Item = V;

    #[inline(always)]
    fn poll_next(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Option<Self::Item>> {
        self.0.poll_recv(cx)
    }
}

impl<T, V> FusedStream for LaneStream<T, V>
where
    T: Key + Unpin,
{
    #[inline(always)]
    fn is_terminated(&self) -> bool {
        self.0.is_closed()
    }
}

impl<T, V> From<LaneRx<T, V>> for LaneStream<T, V>
where
    T: Key + Unpin,
{
    #[inline(always)]
    fn from(value: LaneRx<T, V>) -> Self {
        Self::new(value)
    }
}

impl<T, V> LaneRx<T, V>
where
    T: Key + Unpin,
{
    /// Converts this lane receiver into a [`LaneStream<T, V>`].
    ///
    /// This is only available when the `util` feature is enabled.
    #[inline]
    pub fn into_stream(self) -> LaneStream<T, V> {
        self.into()
    }
}
