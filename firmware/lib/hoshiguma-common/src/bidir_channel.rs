use embassy_sync::{
    blocking_mutex::raw::RawMutex,
    pubsub::{PubSubChannel, Publisher, Subscriber},
};

pub struct BiDirectionalChannel<
    M: RawMutex,
    AToB: Clone,
    BToA: Clone,
    const CAP: usize,
    const NUM_A: usize,
    const NUM_B: usize,
> {
    a_to_b: PubSubChannel<M, AToB, CAP, NUM_B, NUM_A>,
    b_to_a: PubSubChannel<M, BToA, CAP, NUM_A, NUM_B>,
}

impl<
    M: RawMutex,
    AToB: Clone,
    BToA: Clone,
    const CAP: usize,
    const NUM_A: usize,
    const NUM_B: usize,
> BiDirectionalChannel<M, AToB, BToA, CAP, NUM_A, NUM_B>
{
    pub const fn new() -> Self {
        Self {
            a_to_b: PubSubChannel::new(),
            b_to_a: PubSubChannel::new(),
        }
    }

    pub fn side_a<'a>(&'a self) -> Side<'a, M, BToA, AToB, CAP, NUM_A, NUM_B> {
        Side {
            to_me: self.b_to_a.subscriber().unwrap(),
            to_you: self.a_to_b.publisher().unwrap(),
        }
    }

    pub fn side_b<'a>(&'a self) -> Side<'a, M, AToB, BToA, CAP, NUM_B, NUM_A> {
        Side {
            to_me: self.a_to_b.subscriber().unwrap(),
            to_you: self.b_to_a.publisher().unwrap(),
        }
    }
}

pub struct Side<
    'a,
    M: RawMutex,
    ToMe: Clone,
    ToYou: Clone,
    const CAP: usize,
    const NUM_ME: usize,
    const NUM_YOU: usize,
> {
    pub to_me: Subscriber<'a, M, ToMe, CAP, NUM_ME, NUM_YOU>,
    pub to_you: Publisher<'a, M, ToYou, CAP, NUM_YOU, NUM_ME>,
}
