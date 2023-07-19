use tokio::sync::mpsc;
use tokio::sync::mpsc::error::SendError;

use crate::bus::Bus;
use crate::lane::{Lane, LaneTx};
use crate::map::Key;

/// A multiplexer that allows for multiple senders and receivers act on a single
/// stream of messages.
#[derive(Debug)]
pub struct Mux<T: Key, V> {
    bus: Bus<T, V>,
    tx: mpsc::Sender<(T, V)>,
}

impl<T: Key, V> Mux<T, V> {
    /// Create a new [`Mux`](crate::mux::Mux) with the given buffer sizes.
    ///
    /// # Parameters
    /// * `buf` - The buffer size of the underlying incoming messages buffer.
    /// * `lane_buf` - The buffer size of each lane.
    ///
    /// # Returns
    /// A new [`Mux`](crate::mux::Mux) and a receiver that aggregates
    /// incoming messages from all lanes.
    #[inline]
    pub fn new(buf: usize, lane_buf: usize) -> (Self, mpsc::Receiver<(T, V)>) {
        let (tx, rx) = mpsc::channel(buf);
        let mux = Self {
            bus: Bus::new(lane_buf),
            tx,
        };

        (mux, rx)
    }

    /// Send a message to a lane.
    ///
    /// # Parameters
    /// * `tag` - The tag of the lane.
    /// * `value` - The message to send.
    ///
    /// # Returns
    /// * [`Ok(None)`] - If the message is sent to an existing lane.
    /// * [`Ok(Some(lane))`] - If the message is sent to a new lane, the new
    ///  lane  is returned.
    /// * [`Err(SendError((tag, value)))`] - If the lane associated with the
    ///   given tag is closed. closed.
    #[inline]
    pub async fn send(
        &mut self,
        tag: T,
        value: V,
    ) -> Result<Option<Lane<T, V>>, SendError<(T, V)>> {
        self.bus.push(tag, value).await.map(|rx| {
            rx.map(|rx| {
                let tx = LaneTx::new(self.tx.clone(), rx.tag().clone());

                Lane::from_parts(tx, rx)
            })
        })
    }

    /// Close all lanes.
    #[inline]
    pub fn close(self) {
        drop(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn mux_test() {
        let (mut mux_tx, mut mux_rx) = Mux::new(1024, 128);

        tokio::spawn(async move {
            while let Some((tag, msg)) = mux_rx.recv().await {
                println!("<-- tag: {}, msg: {}", tag, msg);
            }

            eprintln!("[!] mux unexpectedly closed, aborting");
        });

        for i in 0..1000 {
            for j in 0..1000 {
                let Some(lane) =
                    (match mux_tx.send(i, format!("hello {i} #{j}")).await {
                        | Ok(lane) => lane,
                        | Err(mpsc::error::SendError((tag, msg))) => {
                            eprintln!(
                                "[!] could not send: tag: {}, msg: {}",
                                tag, msg
                            );
                            break;
                        }
                    })
                else {
                    continue;
                };

                tokio::spawn(async move {
                    let (mut tx, mut rx) = lane.split();

                    while let Some(msg) = rx.recv().await {
                        println!("--> tag: {}, msg: {}", tx.tag(), msg);

                        if tx.send(format!("reply to: {msg}")).await.is_err() {
                            break;
                        }
                    }
                });
            }
        }

        eprintln!("[!] connection closed");
    }
}
