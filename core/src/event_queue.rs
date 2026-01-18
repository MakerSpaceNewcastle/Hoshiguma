use serde::{Deserialize, Serialize};

pub struct TransactionalQueue<E, const N: usize> {
    events: heapless::Deque<E, N>,
    stats: Statistics,
}

impl<E, const N: usize> Default for TransactionalQueue<E, N> {
    fn default() -> Self {
        Self {
            events: heapless::Deque::new(),
            stats: Default::default(),
        }
    }
}

impl<E: Clone, const N: usize> TransactionalQueue<E, N> {
    pub fn is_empty(&self) -> bool {
        self.events.is_empty()
    }

    pub fn len(&self) -> usize {
        self.events.len()
    }

    pub fn statistics(&self) -> Statistics {
        self.stats.clone()
    }

    pub fn push(&mut self, event: E) {
        if self.events.is_full() {
            self.events.pop_front();
            self.stats.discarded = self.stats.discarded.wrapping_add(1);
        }

        assert!(self.events.push_back(event).is_ok());
        self.stats.pushed = self.stats.pushed.wrapping_add(1);
    }

    pub fn pop_request(&mut self) -> RetrievalTransaction {
        let count = core::cmp::min(1, self.events.len());
        RetrievalTransaction { event_count: count }
    }

    pub fn pop_get(&mut self, t: &RetrievalTransaction) -> Option<E> {
        match t.event_count {
            0 => None,
            1 => self.events.front().cloned(),
            _ => panic!("Unexpected event count requested: {}", t.event_count),
        }
    }

    pub fn pop_commit(&mut self, t: RetrievalTransaction) {
        for _ in 0..t.event_count {
            self.events.pop_front();
        }
    }
}

pub struct RetrievalTransaction {
    /// The number of events to be retrieved in the transaction.
    event_count: usize,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "no-std", derive(defmt::Format))]
pub struct Statistics {
    pub pushed: usize,
    pub discarded: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn event_queue_new() {
        let queue: TransactionalQueue<usize, 5> = TransactionalQueue::default();
        assert!(queue.events.is_empty());
        assert_eq!(queue.events.len(), 0);
        assert_eq!(queue.stats.pushed, 0);
        assert_eq!(queue.stats.discarded, 0);
    }

    #[test]
    fn event_queue_push() {
        let mut queue: TransactionalQueue<usize, 2> = TransactionalQueue::default();
        queue.push(1);
        queue.push(2);
        assert!(!queue.events.is_empty());
        assert_eq!(queue.events.len(), 2);
        assert_eq!(queue.stats.pushed, 2);
        assert_eq!(queue.stats.discarded, 0);

        queue.push(3);
        assert_eq!(queue.events.len(), 2);
        assert_eq!(queue.stats.pushed, 3);
        assert_eq!(queue.stats.discarded, 1);
    }

    #[test]
    fn event_queue_ret_request() {
        let mut queue: TransactionalQueue<usize, 5> = TransactionalQueue::default();
        queue.push(1);
        queue.push(2);

        let transaction = queue.pop_request();
        assert_eq!(transaction.event_count, 1);
    }

    #[test]
    fn event_queue_ret_get() {
        let mut queue: TransactionalQueue<usize, 5> = TransactionalQueue::default();
        queue.push(1);
        queue.push(2);

        let transaction = queue.pop_request();
        let event = queue.pop_get(&transaction);
        assert_eq!(event, Some(1));
    }

    #[test]
    fn event_queue_ret_commit() {
        let mut queue: TransactionalQueue<usize, 5> = TransactionalQueue::default();
        queue.push(1);
        queue.push(2);

        let transaction = queue.pop_request();
        queue.pop_commit(transaction);
        assert_eq!(queue.events.len(), 1);
    }
}
