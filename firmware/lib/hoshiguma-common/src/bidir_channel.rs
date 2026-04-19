use embassy_sync::{
    blocking_mutex::raw::RawMutex,
    pubsub::{PubSubChannel, Publisher, Subscriber},
};

pub struct BiDirectionalChannel<
    M: RawMutex,
    T: Clone,
    const CAP: usize,
    const NUM_A: usize,
    const NUM_B: usize,
> {
    a_to_b: PubSubChannel<M, T, CAP, NUM_B, NUM_A>,
    b_to_a: PubSubChannel<M, T, CAP, NUM_A, NUM_B>,
}

impl<M: RawMutex, T: Clone, const CAP: usize, const NUM_A: usize, const NUM_B: usize>
    BiDirectionalChannel<M, T, CAP, NUM_A, NUM_B>
{
    pub const fn new() -> Self {
        Self {
            a_to_b: PubSubChannel::new(),
            b_to_a: PubSubChannel::new(),
        }
    }

    pub fn side_a<'a>(&'a self) -> Side<'a, M, T, CAP, NUM_A, NUM_B> {
        Side {
            outbox: self.a_to_b.publisher().unwrap(),
            inbox: self.b_to_a.subscriber().unwrap(),
        }
    }

    pub fn side_b<'a>(&'a self) -> Side<'a, M, T, CAP, NUM_B, NUM_A> {
        Side {
            outbox: self.b_to_a.publisher().unwrap(),
            inbox: self.a_to_b.subscriber().unwrap(),
        }
    }
}

pub struct Side<
    'a,
    M: RawMutex,
    T: Clone,
    const CAP: usize,
    const NUM_US: usize,
    const NUM_THEM: usize,
> {
    pub outbox: Publisher<'a, M, T, CAP, NUM_THEM, NUM_US>,
    pub inbox: Subscriber<'a, M, T, CAP, NUM_US, NUM_THEM>,
}
