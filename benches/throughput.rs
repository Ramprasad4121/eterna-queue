//! @title MPMC Queue Throughput Benchmarks
//! @author E Ram Prasad
//! @notice This benchmark suite measures the operations-per-second (throughput) of the bounded MPMC queue.
//! @dev Utilizes the `criterion` crate to test symmetric thread configurations (from 1x1 up to 16x16) across varying capacities, as well as an asymmetric backpressure workload (8 producers, 2 consumers).
use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use eterna_queue::{queue::MpmcQueue, BoundedQueue};
use std::sync::Arc;
use std::thread;

/// @notice Executes the complete performance testing suite for the MPMC Queue.
/// @dev Iterates over defined capacities and thread pairs, spawning independent producers and consumers. `std::hint::black_box` is used to prevent the compiler from optimizing away the `pop` operations.
/// @param c The mutable Criterion context used to configure and group the benchmarks.
fn bench_queue(c: &mut Criterion) {
    let mut group = c.benchmark_group("MPMC Throughput");

    let capacities = [64, 256, 1024];
    let thread_pairs = [1, 2, 4, 8, 16];
    let items_per_thread = 5_000;

    // 1. Symmetric Workloads (NxN)
    for &cap in &capacities {
        for &pairs in &thread_pairs {
            // Calculate total operations (pushes) for the throughput metric
            let total_items = (pairs * items_per_thread) as u64;
            group.throughput(Throughput::Elements(total_items));

            group.bench_with_input(
                BenchmarkId::new(format!("Sym_Cap_{}", cap), format!("{}x{}", pairs, pairs)),
                &(pairs, cap),
                |b, &(p, c_cap)| {
                    b.iter(|| {
                        let q = Arc::new(MpmcQueue::new(c_cap));
                        let mut producers = vec![];
                        let mut consumers = vec![];

                        for _ in 0..p {
                            let q_clone = Arc::clone(&q);
                            producers.push(thread::spawn(move || {
                                for i in 0..items_per_thread {
                                    q_clone.push(i);
                                }
                            }));
                        }

                        for _ in 0..p {
                            let q_clone = Arc::clone(&q);
                            consumers.push(thread::spawn(move || {
                                for _ in 0..items_per_thread {
                                    std::hint::black_box(q_clone.pop());
                                }
                            }));
                        }

                        for t in producers {
                            t.join().unwrap();
                        }
                        for t in consumers {
                            t.join().unwrap();
                        }
                    });
                },
            );
        }
    }

    // 2. Asymmetric Workload: 8 Producers, 2 Consumers
    let total_asym_items = 40_000_u64;
    group.throughput(Throughput::Elements(total_asym_items));
    group.bench_function("Asym_8P_2C_Cap256", |b| {
        b.iter(|| {
            let q = Arc::new(MpmcQueue::new(256));
            let mut producers = vec![];
            let mut consumers = vec![];

            let items_per_producer = total_asym_items as usize / 8;
            let items_per_consumer = total_asym_items as usize / 2;

            for _ in 0..8 {
                let q_clone = Arc::clone(&q);
                producers.push(thread::spawn(move || {
                    for i in 0..items_per_producer {
                        q_clone.push(i);
                    }
                }));
            }

            for _ in 0..2 {
                let q_clone = Arc::clone(&q);
                consumers.push(thread::spawn(move || {
                    for _ in 0..items_per_consumer {
                        std::hint::black_box(q_clone.pop());
                    }
                }));
            }

            for t in producers {
                t.join().unwrap();
            }
            for t in consumers {
                t.join().unwrap();
            }
        });
    });

    group.finish();
}

criterion_group!(benches, bench_queue);
criterion_main!(benches);
