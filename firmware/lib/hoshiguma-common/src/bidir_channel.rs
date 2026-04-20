use core::marker::PhantomData;
use embassy_sync::{
    blocking_mutex::raw::RawMutex,
    pubsub::{PubSubChannel, Publisher, Subscriber},
};

pub trait BiDirectionalChannelSides {
    type SideA;
    type SideB;
}

pub struct BiDirectionalChannel<
    'a,
    M: RawMutex,
    AToB: Clone,
    BToA: Clone,
    const CAP: usize,
    const NUM_A: usize,
    const NUM_B: usize,
> {
    a_to_b: PubSubChannel<M, AToB, CAP, NUM_B, NUM_A>,
    b_to_a: PubSubChannel<M, BToA, CAP, NUM_A, NUM_B>,
    _lifetime: PhantomData<&'a ()>,
}

impl<
    'a,
    M: RawMutex + 'a,
    AToB: Clone + 'a,
    BToA: Clone + 'a,
    const CAP: usize,
    const NUM_A: usize,
    const NUM_B: usize,
> BiDirectionalChannelSides for BiDirectionalChannel<'a, M, AToB, BToA, CAP, NUM_A, NUM_B>
{
    type SideA = Side<'a, M, BToA, AToB, CAP, NUM_A, NUM_B>;
    type SideB = Side<'a, M, AToB, BToA, CAP, NUM_B, NUM_A>;
}

impl<
    'a,
    M: RawMutex,
    AToB: Clone,
    BToA: Clone,
    const CAP: usize,
    const NUM_A: usize,
    const NUM_B: usize,
> Default for BiDirectionalChannel<'a, M, AToB, BToA, CAP, NUM_A, NUM_B>
{
    fn default() -> Self {
        Self {
            a_to_b: PubSubChannel::new(),
            b_to_a: PubSubChannel::new(),
            _lifetime: PhantomData,
        }
    }
}

impl<
    'a,
    M: RawMutex,
    AToB: Clone,
    BToA: Clone,
    const CAP: usize,
    const NUM_A: usize,
    const NUM_B: usize,
> BiDirectionalChannel<'a, M, AToB, BToA, CAP, NUM_A, NUM_B>
{
    pub fn side_a(&'a self) -> Side<'a, M, BToA, AToB, CAP, NUM_A, NUM_B> {
        Side {
            to_me: self
                .b_to_a
                .subscriber()
                .expect("Failed to create subscriber for side A"),
            to_you: self
                .a_to_b
                .publisher()
                .expect("Failed to create publisher for side A"),
        }
    }

    pub fn side_b(&'a self) -> Side<'a, M, AToB, BToA, CAP, NUM_B, NUM_A> {
        Side {
            to_me: self
                .a_to_b
                .subscriber()
                .expect("Failed to create subscriber for side B"),
            to_you: self
                .b_to_a
                .publisher()
                .expect("Failed to create publisher for side B"),
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
