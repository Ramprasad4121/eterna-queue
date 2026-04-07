# Eterna Labs: Bounded MPMC Queue

[![Rust](https://img.shields.io/badge/rust-1.70%2B-orange.svg)](https://www.rust-lang.org)
[![Dependencies](https://img.shields.io/badge/dependencies-std%20only-brightgreen.svg)]()
[![Status](https://img.shields.io/badge/status-benchmarked-blue.svg)]()

A high-throughput, strictly bounded Multi-Producer Multi-Consumer (MPMC) queue implemented in 100% safe Rust using only the standard library (`std`). This design prioritizes correctness, memory safety, and predictable backpressure over lock-free complexity.

---


##  Setup

```bash
git clone https://github.com/Ramprasad4121/eterna-queue.git
cd eterna-queue
```

---

##  Build

```bash
cargo build
```

Release build:

```bash
cargo build --release
```

---

##  Run Tests

```bash
cargo test
```

---

##  Run Benchmarks

```bash
cargo bench
```

---

##  Key Features

* 100% Safe Rust (no `unsafe`)
* Bounded queue with blocking semantics
* No busy-waiting (Condvar-based)
* Backpressure-aware under load
* Stress-tested with 1M+ operations

---

##  Benchmark Results

*Hardware: MacBook Air (Apple Silicon)*

| Workload  | Capacity 64     | Capacity 256    | Capacity 1024       |
| --------- | --------------- | --------------- | ------------------- |
| **1x1**   | 15.08 M ops/sec | 21.80 M ops/sec | **25.42 M ops/sec** |
| **2x2**   | 7.74 M ops/sec  | 11.42 M ops/sec | 14.79 M ops/sec     |
| **4x4**   | 2.99 M ops/sec  | 6.58 M ops/sec  | 9.05 M ops/sec      |
| **8x8**   | 2.29 M ops/sec  | 5.71 M ops/sec  | 8.65 M ops/sec      |
| **16x16** | 1.68 M ops/sec  | 4.89 M ops/sec  | 6.34 M ops/sec      |

**Asymmetric (8P / 2C, Cap 256):** ~2.95 M ops/sec

---

##  Observations

* Larger capacity reduces blocking and improves throughput
* Higher thread counts increase mutex contention
* Asymmetric workloads demonstrate backpressure

---

##  Design Tradeoffs

**Pros**

* Simple and correct
* No busy-waiting
* Efficient under contention

**Cons**

* Mutex limits scalability at high concurrency
* Not ideal for ultra-low latency

---

##  Future Work

* Lock-free ring buffer using atomics
* Cache-line padding to avoid false sharing

---

##  Notes

* Capacity is enforced logically; `VecDeque` is not inherently bounded
* Design prioritizes predictable latency and correctness over peak throughput

