use core::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};
use embassy_time::{Duration, Instant};

#[must_use = "futures do nothing unless you `.await` or poll them"]
pub struct MaybeTimer {
    expires_at: Option<Instant>,
    yielded_once: bool,
}

impl MaybeTimer {
    pub fn at(expires_at: Option<Instant>) -> Self {
        Self {
            expires_at,
            yielded_once: false,
        }
    }
}

impl Unpin for MaybeTimer {}

impl Future for MaybeTimer {
    type Output = ();

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if let Some(expires_at) = self.expires_at {
            if self.yielded_once && expires_at <= Instant::now() {
                Poll::Ready(())
            } else {
                embassy_time_queue_driver::schedule_wake(expires_at.as_ticks(), cx.waker());
                self.yielded_once = true;
                Poll::Pending
            }
        } else {
            // This can be quite long as the future will never complete
            let wake_deadline = Instant::now() + Duration::from_secs(5);
            embassy_time_queue_driver::schedule_wake(wake_deadline.as_ticks(), cx.waker());
            self.yielded_once = true;
            Poll::Pending
        }
    }
}
