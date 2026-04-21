use core::marker::PhantomData;
use defmt::warn;
use embassy_sync::{
    blocking_mutex::raw::RawMutex,
    channel::{Channel, Receiver, Sender},
};

pub trait BiDirectionalChannelSides {
    type SideA;
    type SideB;
}

pub struct BiDirectionalChannel<'a, M: RawMutex, AToB: Clone, BToA: Clone> {
    a_to_b: Channel<M, AToB, 1>,
    b_to_a: Channel<M, BToA, 1>,
    _lifetime: PhantomData<&'a ()>,
}

impl<'a, M: RawMutex + 'a, AToB: Clone + 'a, BToA: Clone + 'a> BiDirectionalChannelSides
    for BiDirectionalChannel<'a, M, AToB, BToA>
{
    type SideA = Side<'a, M, BToA, AToB>;
    type SideB = Side<'a, M, AToB, BToA>;
}

impl<'a, M: RawMutex, AToB: Clone, BToA: Clone> Default
    for BiDirectionalChannel<'a, M, AToB, BToA>
{
    fn default() -> Self {
        Self {
            a_to_b: Channel::new(),
            b_to_a: Channel::new(),
            _lifetime: PhantomData,
        }
    }
}

impl<'a, M: RawMutex, AToB: Clone, BToA: Clone> BiDirectionalChannel<'a, M, AToB, BToA> {
    pub fn side_a(&'a self) -> Side<'a, M, BToA, AToB> {
        Side {
            to_me: self.b_to_a.receiver(),
            to_you: self.a_to_b.sender(),
        }
    }

    pub fn side_b(&'a self) -> Side<'a, M, AToB, BToA> {
        Side {
            to_me: self.a_to_b.receiver(),
            to_you: self.b_to_a.sender(),
        }
    }
}

pub struct Side<'a, M: RawMutex, ToMe: Clone, ToYou: Clone> {
    to_me: Receiver<'a, M, ToMe, 1>,
    to_you: Sender<'a, M, ToYou, 1>,
}

impl<'a, M: RawMutex, ToMe: Clone, ToYou: Clone> Side<'a, M, ToMe, ToYou> {
    pub async fn send(&self, v: ToYou) {
        if !self.to_me.is_empty() {
            warn!("Was about to send a request when the response channel was not empty");
            self.to_me.clear();
        }
        self.to_you.send(v).await;
    }

    pub async fn receive(&self) -> ToMe {
        self.to_me.receive().await
    }
}
