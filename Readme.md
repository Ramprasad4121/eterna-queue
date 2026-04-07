# Eterna Labs: Bounded MPMC Queue

[![Rust](https://img.shields.io/badge/rust-1.70%2B-orange.svg)](https://www.rust-lang.org)
[![Dependencies](https://img.shields.io/badge/dependencies-std%20only-brightgreen.svg)]()
[![Status](https://img.shields.io/badge/status-benchmarked-blue.svg)]()

A high-throughput, strictly bounded Multi-Producer Multi-Consumer (MPMC) queue engineered in 100% safe Rust. Built exclusively with the standard library (`std`), this implementation prioritizes absolute correctness, memory safety, and native backpressure handling over CPU-intensive lock-free spinning.

##  Key Features
*  **100% Safe Rust:** Zero `unsafe` blocks, eliminating the risk of subtle ABA problems or memory ordering bugs.
*  **Guaranteed Integrity:** Mathematically validated under extreme contention (1M+ concurrent items) with zero data loss or duplication.
*  **Zero Busy-Waiting:** Utilizes OS-level thread parking (`Condvar`) to conserve CPU cycles when the queue is blocked.
*  **Native Backpressure:** Safely throttles asymmetric workloads when producers outpace consumers.

---

##  Quick Start & Validation

### Correctness Tests
The test suite ensures robust behavior under extreme contention. Our flagship stress test (`test_high_contention_stress_and_data_loss`) spawns 16 concurrent threads (8P/8C) to push/pop 1,000,000 items. A strict summation asserts 100% data integrity.
```bash
cargo test
```

### Throughput Benchmarks
Benchmarks are powered by `criterion`. The suite evaluates symmetric workloads (1x1 up to 16x16), varying buffer capacities (64, 256, 1024), and asymmetric backpressure configurations.
```bash
cargo bench
```

---

##  Benchmark Results

*Hardware: MacBook Air (Apple Silicon). Operations measured in Millions of Elements per second (M ops/sec).*

| Workload | Capacity 64 | Capacity 256 | Capacity 1024 |
|----------|-------------|--------------|---------------|
| **1x1** | 15.08 M ops/sec *(331 µs)* | 21.80 M ops/sec *(229 µs)* |  **25.42 M ops/sec** *(196 µs)* |
| **2x2** | 7.74 M ops/sec *(1.29 ms)* | 11.42 M ops/sec *(875 µs)* | 14.79 M ops/sec *(676 µs)* |
| **4x4** | 2.99 M ops/sec *(6.67 ms)* | 6.58 M ops/sec *(3.03 ms)* | 9.05 M ops/sec *(2.20 ms)* |
| **8x8** | 2.29 M ops/sec *(17.45 ms)*| 5.71 M ops/sec *(7.00 ms)* | 8.65 M ops/sec *(4.62 ms)* |
| **16x16**| 1.68 M ops/sec *(47.60 ms)*| 4.89 M ops/sec *(16.33 ms)*| 6.34 M ops/sec *(12.61 ms)*|

> **Asymmetric Workload (8 Producers, 2 Consumers | Cap 256):** > *Throughput: 2.95 M ops/sec (13.51 ms)*

---

##  Architectural Insights & Observations

### 1. Performance Under Contention
The benchmark data reveals expected system behaviors inherent to a `Mutex`-backed implementation:
* **Capacity Scaling:** Throughput drastically improves with larger queue capacities. At 1x1, scaling from a capacity of 64 to 1024 yields a **~68% increase in throughput**. A larger buffer reduces the frequency at which the queue hits bounds (empty/full), directly minimizing `Condvar::wait()` calls and expensive kernel context switches.
* **Lock Contention Ceiling:** Performance linearly degrades at extremely high thread counts (e.g., 16x16 drops to 6.34 M ops/sec). 32 threads fighting for a single lock forces the OS scheduler to thrash, highlighting the natural ceiling of lock-based synchronization.

### 2. The Asymmetric Regression (Backpressure)
The asymmetric workload (8P/2C) showed a noticeable throughput regression (~2.95 M ops/sec). This is an expected demonstration of **backpressure**. Because 8 producers vastly outpace 2 consumers, the queue stays perpetually full. Producers are constantly forced into a blocking state via `Condvar::wait()`. The overhead of repeatedly waking and sleeping threads dominates the CPU time, throttling the producers to match the consumer ingestion rate.

### 3. Tradeoffs of the Current Design
*  **Pros:** Utilizing a standard `Mutex` and `Condvar` guarantees absolute memory safety, strictly prevents ABA problems, completely avoids CPU-burning busy-waiting, and handles backpressure naturally.
*  **Cons:** Not optimal for ultra-low latency scenarios (sub-microsecond) because thread synchronization must pass through the OS kernel, incurring unavoidable scheduling overhead.

---

##  Future Work: Institutional-Grade Latency
To eliminate lock contention and achieve the true low-latency scaling required for hot-path market data pipelines, the next iteration of this infrastructure would require:

1. **Lock-Free Architecture:** Transitioning to a circular buffer utilizing `AtomicUsize` for head/tail indices with explicit `Acquire`/`Release` memory orderings.
2. **False Sharing Prevention:** Wrapping the head and tail indices in `crossbeam_utils::CachePadded` to ensure they sit on separate CPU cache lines, preventing cores from constantly invalidating each other's L1 cache during concurrent reads and writes.