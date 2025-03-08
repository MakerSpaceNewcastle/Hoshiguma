use serde::{Deserialize, Serialize};

/// A queue that holds events with a fixed capacity.
///
/// # Type Parameters
/// - `E`: The type of events stored in the queue.
/// - `N`: The maximum number of events the queue can hold.
pub struct EventQueue<E, const N: usize, const R: usize> {
    events: heapless::Deque<E, N>,
    stats: EventStatistics,
}

impl<E, const N: usize, const R: usize> Default for EventQueue<E, N, R> {
    fn default() -> Self {
        Self {
            events: heapless::Deque::new(),
            stats: Default::default(),
        }
    }
}

impl<E: Clone, const N: usize, const R: usize> EventQueue<E, N, R> {
    /// Checks if the event queue is empty.
    pub fn is_empty(&self) -> bool {
        self.events.is_empty()
    }

    /// Returns the number of events in the queue.
    pub fn len(&self) -> usize {
        self.events.len()
    }

    /// Returns the statistics of the events in the queue.
    pub fn statistics(&self) -> EventStatistics {
        self.stats.clone()
    }

    /// Pushes an event to the back of the queue.
    pub fn push(&mut self, event: E) {
        if self.events.is_full() {
            self.events.pop_front();
            self.stats.discarded = self.stats.discarded.wrapping_add(1);
        }

        assert!(self.events.push_back(event).is_ok());
        self.stats.total = self.stats.total.wrapping_add(1);
    }

    /// Creates a retrieval transaction for a specified number of events.
    ///
    /// # Parameters
    /// - `count`: The number of events to retrieve.
    ///
    /// # Returns
    /// - A `RetrievalTransaction` containing the number of events to be retrieved.
    pub fn ret_request(&mut self, count: usize) -> RetrievalTransaction {
        let count = core::cmp::min(count, R);
        let count = core::cmp::min(count, self.events.len());
        RetrievalTransaction { event_count: count }
    }

    /// Retrieves events based on the provided transaction.
    ///
    /// # Parameters
    /// - `t`: The retrieval transaction specifying the number of events to retrieve.
    ///
    /// # Returns
    /// - A vector containing the retrieved events.
    pub fn ret_get(&mut self, t: &RetrievalTransaction) -> heapless::Vec<E, R> {
        let mut result = heapless::Vec::new();
        for event in self.events.iter().take(t.event_count) {
            result.push(event.clone()).map_err(|_| ()).unwrap();
        }
        result
    }

    /// Commits the retrieval transaction by removing the specified number of events from the queue.
    ///
    /// # Parameters
    /// - `t`: The retrieval transaction specifying the number of events to remove.
    pub fn ret_commit(&mut self, t: RetrievalTransaction) {
        for _ in 0..t.event_count {
            self.events.pop_front();
        }
    }
}

/// A struct representing a transaction for retrieving events from the queue.
pub struct RetrievalTransaction {
    /// The number of events to be retrieved in the transaction.
    event_count: usize,
}

/// Statistics for events in the queue.
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "no-std", derive(defmt::Format))]
pub struct EventStatistics {
    /// Total number of events processed.
    pub total: usize,

    /// Number of events discarded due to queue overflow.
    pub discarded: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone, Debug, PartialEq)]
    struct TestEvent {
        id: usize,
    }

    #[test]
    fn test_event_queue_new() {
        let queue: EventQueue<TestEvent, 5, 8> = EventQueue::default();
        assert!(queue.events.is_empty());
        assert_eq!(queue.events.len(), 0);
        assert_eq!(queue.stats.total, 0);
        assert_eq!(queue.stats.discarded, 0);
    }

    #[test]
    fn test_event_queue_push() {
        let mut queue: EventQueue<TestEvent, 2, 8> = EventQueue::default();
        queue.push(TestEvent { id: 1 });
        queue.push(TestEvent { id: 2 });
        assert!(!queue.events.is_empty());
        assert_eq!(queue.events.len(), 2);
        assert_eq!(queue.stats.total, 2);
        assert_eq!(queue.stats.discarded, 0);

        // Push another event to test discard logic
        queue.push(TestEvent { id: 3 });
        assert_eq!(queue.events.len(), 2);
        assert_eq!(queue.stats.total, 3);
        assert_eq!(queue.stats.discarded, 1);
    }

    #[test]
    fn test_event_queue_ret_request() {
        let mut queue: EventQueue<TestEvent, 5, 8> = EventQueue::default();
        queue.push(TestEvent { id: 1 });
        queue.push(TestEvent { id: 2 });

        let transaction = queue.ret_request(1);
        assert_eq!(transaction.event_count, 1);

        let transaction = queue.ret_request(3);
        assert_eq!(transaction.event_count, 2);
    }

    #[test]
    fn test_event_queue_ret_request_above_limit() {
        let mut queue: EventQueue<TestEvent, 5, 3> = EventQueue::default();
        queue.push(TestEvent { id: 1 });
        queue.push(TestEvent { id: 2 });
        queue.push(TestEvent { id: 3 });
        queue.push(TestEvent { id: 4 });
        queue.push(TestEvent { id: 5 });

        let transaction = queue.ret_request(5);
        assert_eq!(transaction.event_count, 3);
    }

    #[test]
    fn test_event_queue_ret_get() {
        let mut queue: EventQueue<TestEvent, 5, 8> = EventQueue::default();
        queue.push(TestEvent { id: 1 });
        queue.push(TestEvent { id: 2 });

        let transaction = queue.ret_request(2);
        let events = queue.ret_get(&transaction);
        assert_eq!(events.len(), 2);
        assert_eq!(events[0], TestEvent { id: 1 });
        assert_eq!(events[1], TestEvent { id: 2 });
    }

    #[test]
    fn test_event_queue_ret_commit() {
        let mut queue: EventQueue<TestEvent, 5, 8> = EventQueue::default();
        queue.push(TestEvent { id: 1 });
        queue.push(TestEvent { id: 2 });

        let transaction = queue.ret_request(2);
        queue.ret_commit(transaction);
        assert_eq!(queue.events.len(), 0);
    }
}
