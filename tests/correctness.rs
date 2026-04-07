use eterna_queue::queue::MpmcQueue;
use eterna_queue::BoundedQueue;
use std::sync::Arc;
use std::thread;

// Verifies that the non-blocking methods correctly return Option/Result
// and do not inadvertently put the thread to sleep.
#[test]
fn test_try_push_pop_semantics() {
    let q = MpmcQueue::new(2);

    // Queue is empty, should return None
    assert_eq!(q.try_pop(), None);

    // Fill the queue
    assert_eq!(q.try_push(10), Ok(()));
    assert_eq!(q.try_push(20), Ok(()));

    // Queue is at capacity, should return the item in the Err variant
    assert_eq!(q.try_push(30), Err(30));

    // Empty the queue and verify FIFO ordering
    assert_eq!(q.try_pop(), Some(10));
    assert_eq!(q.try_pop(), Some(20));
    assert_eq!(q.try_pop(), None);
}

// Tests the absolute minimum bounds of the queue.
// A capacity of 1 forces immediate contention and backpressure,
// ensuring the Condvar wait/notify logic triggers correctly on every single operation.
#[test]
fn test_capacity_one_edge_case() {
    let q = Arc::new(MpmcQueue::new(1));
    let q_clone = Arc::clone(&q);

    let handle = thread::spawn(move || {
        q_clone.push(100);

        // This second push is guaranteed to block the producer thread
        // until the consumer thread pops the first item.
        q_clone.push(200);
    });

    assert_eq!(q.pop(), 100);
    assert_eq!(q.pop(), 200);
    handle.join().unwrap();
}

// The ultimate stress test.
// Spawns 16 concurrent threads to process 1,000,000 items through a tiny buffer (64).
// This forces extreme lock contention and continuous thread sleeping/waking.
#[test]
fn test_high_contention_stress_and_data_loss() {
    let capacity = 64;
    let q = Arc::new(MpmcQueue::new(capacity));

    let num_producers = 8;
    let num_consumers = 8;
    let items_per_producer = 125_000;

    let mut producers = vec![];
    let mut consumers = vec![];

    // Spawn Producers: We track the mathematical sum of every item pushed
    // to establish a source of truth for data integrity.
    for i in 0..num_producers {
        let q_clone = Arc::clone(&q);
        producers.push(thread::spawn(move || {
            let mut produced_sum: u64 = 0;
            for j in 0..items_per_producer {
                let val = (i * items_per_producer + j) as u64;
                q_clone.push(val);
                produced_sum += val;
            }
            produced_sum
        }));
    }

    // Spawn Consumers: We sum up every item received from the queue.
    for _ in 0..num_consumers {
        let q_clone = Arc::clone(&q);
        consumers.push(thread::spawn(move || {
            let mut consumed_sum: u64 = 0;
            for _ in 0..items_per_producer {
                consumed_sum += q_clone.pop();
            }
            consumed_sum
        }));
    }

    // Aggregate expected vs actual sums
    let expected_total_sum: u64 = producers.into_iter().map(|p| p.join().unwrap()).sum();
    let actual_total_sum: u64 = consumers.into_iter().map(|c| c.join().unwrap()).sum();

    // If these sums match perfectly, it mathematically proves:
    // 1. No items were dropped.
    // 2. No items were duplicated.
    // 3. No threads permanently deadlocked.
    assert_eq!(
        expected_total_sum, actual_total_sum,
        "CRITICAL: Data loss or duplication detected"
    );

    // Ensure no ghost items remain
    assert_eq!(q.try_pop(), None);
}
