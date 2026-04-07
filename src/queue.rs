//! @title A bounded, Multi-Producer Multi-Consumer (MPMC) queue.
//! @author: E Ram Prasad
//! @notice This structure provides a thread-safe queue with a fixed maximum capacity.
//! @dev This implementation prioritizes memory safety and correctness over lock-free operations. It utilizes a `Mutex` to protect the underlying `VecDeque` and `Condvar`s to handle synchronization without CPU-intensive busy-waiting.

use crate::BoundedQueue;
use std::collections::VecDeque;
use std::sync::{Condvar, Mutex};

pub struct MpmcQueue<T> {
    inner: Mutex<Inner<T>>,
    not_full: Condvar,
    not_empty: Condvar,
}

/// @title Internal state for the MPMC Queue.
/// @dev Encapsulates the circular buffer and its maximum capacity, designed to be wrapped within the primary Mutex.
struct Inner<T> {
    buffer: VecDeque<T>,
    capacity: usize,
}

impl<T: Send> BoundedQueue<T> for MpmcQueue<T> {
    /// @notice Initializes a new MpmcQueue with the specified maximum capacity.
    /// @dev Panics if the provided capacity is zero. Bounded queues must have a capacity of at least 1.
    /// @param capacity The maximum number of elements the queue can hold at any given time.
    /// @return A newly constructed MpmcQueue instance.
    fn new(capacity: usize) -> Self {
        assert!(capacity > 0, "Queue capacity must be greater than zero");
        Self {
            inner: Mutex::new(Inner {
                buffer: VecDeque::with_capacity(capacity),
                capacity,
            }),
            not_full: Condvar::new(),
            not_empty: Condvar::new(),
        }
    }

    /// @notice Pushes an item into the queue. Blocks the calling thread if the queue is full.
    /// @dev A `while` loop is used with `not_full.wait()` to securely guard against spurious wakeups. The mutex lock is explicitly dropped before `notify_one()` is called to prevent the woken thread from immediately blocking again.
    /// @param item The element of type T to be enqueued.
    fn push(&self, item: T) {
        let mut lock = self.inner.lock().unwrap();

        // Wait while the queue is at capacity
        while lock.buffer.len() == lock.capacity {
            lock = self.not_full.wait(lock).unwrap();
        }

        lock.buffer.push_back(item);

        // Drop the lock before notifying to prevent the woken thread
        // from immediately blocking on the mutex again
        drop(lock);
        self.not_empty.notify_one();
    }

    /// @notice Pops an item from the front of the queue. Blocks the calling thread if the queue is empty.
    /// @dev A `while` loop is used with `not_empty.wait()` to securely guard against spurious wakeups.
    /// @return The oldest item in the queue.
    fn pop(&self) -> T {
        let mut lock = self.inner.lock().unwrap();

        // Wait while the queue is empty
        while lock.buffer.is_empty() {
            lock = self.not_empty.wait(lock).unwrap();
        }

        let item = lock.buffer.pop_front().unwrap();

        // Drop the lock before notifying
        drop(lock);
        self.not_full.notify_one();

        item
    }

    /// @notice Attempts to push an item into the queue without blocking the calling thread.
    /// @dev If the queue is at capacity, the lock is released and an Error containing the item is returned to the caller.
    /// @param item The element of type T to be enqueued.
    /// @return Ok(()) if the push was successful, or Err(item) returning the original item if the queue was full.
    fn try_push(&self, item: T) -> Result<(), T> {
        let mut lock = self.inner.lock().unwrap();

        if lock.buffer.len() == lock.capacity {
            Err(item)
        } else {
            lock.buffer.push_back(item);
            drop(lock);
            self.not_empty.notify_one();
            Ok(())
        }
    }

    /// @notice Attempts to pop an item from the queue without blocking the calling thread.
    /// @dev Returns None immediately if the queue is empty, avoiding any Condvar wait states.
    /// @return Some(T) containing the oldest item if successful, or None if the queue was empty.
    fn try_pop(&self) -> Option<T> {
        let mut lock = self.inner.lock().unwrap();

        if lock.buffer.is_empty() {
            None
        } else {
            let item = lock.buffer.pop_front().unwrap();
            drop(lock);
            self.not_full.notify_one();
            Some(item)
        }
    }
}
