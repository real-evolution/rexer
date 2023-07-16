/// A oneshot channel sender side.
#[derive(Debug)]
pub struct OneshotProducer<T>(kanal::OneshotAsyncSender<T>);

impl<T> OneshotProducer<T> {
    /// Sends a value to the consumer, consuming this sender.
    ///
    /// The value is returned back if the value could not be sent.
    ///
    /// # Parameters
    /// - `value`: The value to send.
    #[inline]
    pub async fn send(self, value: T) -> Result<(), T> {
        self.0.send(value).await
    }
}

/// A oneshot channel consumer side.
#[derive(Debug)]
pub struct OneshotConsumer<T>(kanal::OneshotAsyncReceiver<T>);

impl<T> OneshotConsumer<T> {
    /// Receives a value from the producer.
    ///
    /// # Returns
    /// The received value.
    #[inline]
    pub async fn recv(self) -> Result<T, super::ConsumeError> {
        self.0.recv().await.map_err(Into::into)
    }
}

#[inline]
#[must_use]
pub fn oneshot<T>() -> (OneshotProducer<T>, OneshotConsumer<T>) {
    let (sender, receiver) = kanal::oneshot_async();

    (OneshotProducer(sender), OneshotConsumer(receiver))
}
