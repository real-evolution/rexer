use std::sync::atomic::{AtomicUsize, Ordering};

use tokio::sync::mpsc;

/// A bounded, single-producer, single-consumer queue.
///
/// This is a thin wrapper around [`tokio::sync::mpsc::channel`] that emulates
/// an async work-queue behaviour.
#[derive(Debug)]
pub struct Queue<T> {
    tx: mpsc::Sender<T>,
    rx: mpsc::Receiver<T>,
    len: AtomicUsize,
    cap: usize,
}

impl<T> Queue<T> {
    /// Creates a new, bounded queue.
    ///
    /// # Parameters
    /// * `capacity` - The capacity of the queue.
    #[inline]
    pub fn new_bounded(capacity: usize) -> Self {
        let (tx, rx) = mpsc::channel(capacity);

        Self {
            tx,
            rx,
            len: AtomicUsize::new(0),
            cap: capacity,
        }
    }

    /// Pushes an item to the back of the queue.
    ///
    /// If the queue is full, this will asynchronously wait until there is space
    /// available.
    ///
    /// # Parameters
    /// * `item` - The item to push to the back of the queue.
    #[inline]
    pub async fn push_back(&self, item: T) {
        self.tx.send(item).await.unwrap();
        self.len.fetch_add(1, Ordering::Relaxed);
    }

    /// Pops an item from the front of the queue.
    #[inline]
    pub async fn pop_front(&mut self) -> T {
        let item = self.rx.recv().await.unwrap();
        self.len.fetch_sub(1, Ordering::Relaxed);
        item
    }

    /// Gets the number of items currently present in the queue.
    #[inline]
    pub fn len(&self) -> usize {
        self.len.load(Ordering::Relaxed)
    }

    /// Gets whether the queue is empty or not.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Gets whether the queue is full or not.
    #[inline]
    pub fn is_full(&self) -> bool {
        self.len() == self.capacity()
    }

    /// Gets the full capacity of the queue.
    #[inline]
    pub const fn capacity(&self) -> usize {
        self.cap
    }
}
