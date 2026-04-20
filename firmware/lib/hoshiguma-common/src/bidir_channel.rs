use core::marker::PhantomData;
use embassy_sync::{
    blocking_mutex::raw::RawMutex,
    channel::{Channel, Receiver, Sender},
};

pub trait BiDirectionalChannelSides {
    type SideA;
    type SideB;
}

pub struct BiDirectionalChannel<'a, M: RawMutex, AToB: Clone, BToA: Clone, const CAP: usize> {
    a_to_b: Channel<M, AToB, CAP>,
    b_to_a: Channel<M, BToA, CAP>,
    _lifetime: PhantomData<&'a ()>,
}

impl<'a, M: RawMutex + 'a, AToB: Clone + 'a, BToA: Clone + 'a, const CAP: usize>
    BiDirectionalChannelSides for BiDirectionalChannel<'a, M, AToB, BToA, CAP>
{
    type SideA = Side<'a, M, BToA, AToB, CAP>;
    type SideB = Side<'a, M, AToB, BToA, CAP>;
}

impl<'a, M: RawMutex, AToB: Clone, BToA: Clone, const CAP: usize> Default
    for BiDirectionalChannel<'a, M, AToB, BToA, CAP>
{
    fn default() -> Self {
        Self {
            a_to_b: Channel::new(),
            b_to_a: Channel::new(),
            _lifetime: PhantomData,
        }
    }
}

impl<'a, M: RawMutex, AToB: Clone, BToA: Clone, const CAP: usize>
    BiDirectionalChannel<'a, M, AToB, BToA, CAP>
{
    pub fn side_a(&'a self) -> Side<'a, M, BToA, AToB, CAP> {
        Side {
            to_me: self.b_to_a.receiver(),
            to_you: self.a_to_b.sender(),
        }
    }

    pub fn side_b(&'a self) -> Side<'a, M, AToB, BToA, CAP> {
        Side {
            to_me: self.a_to_b.receiver(),
            to_you: self.b_to_a.sender(),
        }
    }
}

pub struct Side<'a, M: RawMutex, ToMe: Clone, ToYou: Clone, const CAP: usize> {
    pub to_me: Receiver<'a, M, ToMe, CAP>,
    pub to_you: Sender<'a, M, ToYou, CAP>,
}
