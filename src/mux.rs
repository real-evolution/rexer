use tokio::sync::mpsc;

use crate::{Bus, Key, Lane, LaneTx};

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
    pub async fn send(&mut self, tag: T, value: V) -> Option<Lane<T, V>> {
        self.bus.push(tag, value).await.map(|rx| {
            let tx = LaneTx::new(self.tx.clone(), rx.tag().clone());

            Lane::from_parts(tx, rx)
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
    use fake::{Fake, Faker};
    use rand::rngs::StdRng;
    use rand::SeedableRng;

    use super::*;

    #[tokio::test]
    async fn mux_test() {
        let buf: usize = (1..32).fake();
        let lane_buf: usize = (1..8).fake();
        let lane_cnt: u64 = (10..50).fake();
        let msg_cnt: u64 = (10..50).fake();

        let (mut mux_tx, mut mux_rx) = Mux::new(buf, lane_buf);

        let pull = tokio::spawn(async move {
            for lane_no in 0..lane_cnt {
                let rng = &mut get_rng(lane_no);

                for msg_no in 0..msg_cnt {
                    let (actual_tag, actual_msg) = mux_rx.recv().await.unwrap();

                    assert_eq!(lane_no, actual_tag);

                    let (actual_msg_no, actual_msg): (u64, String) = actual_msg;
                    let expected_msg: String = Faker.fake_with_rng(rng);

                    assert_eq!(msg_no, actual_msg_no);
                    assert_eq!(expected_msg, actual_msg);
                }
            }

            assert_eq!(None, mux_rx.recv().await);
        });

        for lane_no in 0..lane_cnt {
            let rng = &mut get_rng(lane_no);

            for msg_no in 0..msg_cnt {
                let msg: String = Faker.fake_with_rng(rng);
                let lane = mux_tx.send(lane_no, (msg_no, msg.clone())).await;

                if msg_no == 0 {
                    tokio::spawn(handle_lane(lane.unwrap(), msg_cnt));
                }
            }
        }

        async fn handle_lane(lane: Lane<u64, (u64, String)>, msg_cnt: u64) {
            let (mut tx, mut rx) = lane.split();
            let rng = &mut get_rng(*tx.tag());

            for msg_no in 0..msg_cnt {
                let (actual_msg_no, actual_msg) = rx.recv().await.unwrap();
                let expected_msg: String = Faker.fake_with_rng(rng);

                assert_eq!(msg_no, actual_msg_no);
                assert_eq!(expected_msg, actual_msg);

                tx.send((msg_no, expected_msg)).await.unwrap();
            }

            assert_eq!(None, rx.recv().await);
        }

        mux_tx.close();
        pull.await.unwrap();
    }

    #[inline]
    fn get_rng(lane: u64) -> StdRng {
        union Seed {
            u8: [u8; 32],
            u64: [u64; 4],
        }

        let seed = Seed {
            u64: [lane, lane, lane, lane],
        };

        StdRng::from_seed(unsafe { seed.u8 })
    }
}
