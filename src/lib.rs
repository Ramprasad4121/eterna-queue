pub mod queue;

/// @title Bounded MPMC Queue Trait
/// @author E Ram Prasad
/// @notice Defines the standard interface for a bounded Multi-Producer Multi-Consumer queue.
/// @dev Any implementation of this trait must be thread-safe (`Send + Sync`) and correctly handle concurrent pushes and pops respecting the specified maximum capacity.
pub trait BoundedQueue<T: Send>: Send + Sync {
    /// @notice Initializes a new bounded queue with the specified capacity.
    /// @dev Implementations should enforce that the capacity is greater than 0.
    /// @param capacity The maximum number of elements the queue can hold at any given time.
    /// @return A newly constructed instance of the queue.
    fn new(capacity: usize) -> Self
    where
        Self: Sized;

    /// @notice Pushes an item into the queue. Blocks the calling thread if the queue is currently full.
    /// @param item The element of type T to be enqueued.
    fn push(&self, item: T);

    /// @notice Pops an item from the queue. Blocks the calling thread if the queue is currently empty.
    /// @return The oldest item of type T from the queue.
    fn pop(&self) -> T;

    /// @notice Attempts to push an item into the queue without blocking the calling thread.
    /// @param item The element of type T to be enqueued.
    /// @return `Ok(())` if the item was successfully pushed, or `Err(item)` returning the original item if the queue is at capacity.
    fn try_push(&self, item: T) -> Result<(), T>;

    /// @notice Attempts to pop an item from the queue without blocking the calling thread.
    /// @return `Some(T)` containing the oldest item if successful, or `None` if the queue is currently empty.
    fn try_pop(&self) -> Option<T>;
}
