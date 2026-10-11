use parallel_histogram::{histogram_parallel, histogram_sequential};
use std::hint::black_box;
use std::time::{Duration, Instant};

fn elapsed_per_run(mut operation: impl FnMut(), iterations: usize) -> Duration {
    let start = Instant::now();
    for _ in 0..iterations {
        operation();
    }
    start.elapsed() / iterations as u32
}

fn main() {
    let byte_count = std::env::var("HISTOGRAM_BYTES")
        .ok()
        .and_then(|value| value.parse().ok())
        .unwrap_or(4 * 1024 * 1024usize);
    let iterations = std::env::var("HISTOGRAM_ITERATIONS")
        .ok()
        .and_then(|value| value.parse().ok())
        .unwrap_or(5usize)
        .max(1);
    let input: Vec<u8> = (0..byte_count)
        .map(|index| ((index.wrapping_mul(73) ^ (index >> 3)) & 0xff) as u8)
        .collect();
    let available_workers = std::thread::available_parallelism()
        .map(usize::from)
        .unwrap_or(1);
    let mut worker_counts = vec![1, 2, 4, available_workers];
    worker_counts.sort_unstable();
    worker_counts.dedup();

    let sequential = elapsed_per_run(
        || {
            black_box(histogram_sequential(black_box(&input)));
        },
        iterations,
    );
    println!(
        "input_bytes={byte_count} iterations={iterations} sequential={:.3} ms ({:.1} MiB/s)",
        sequential.as_secs_f64() * 1_000.0,
        byte_count as f64 / sequential.as_secs_f64() / (1024.0 * 1024.0)
    );

    for workers in worker_counts {
        let elapsed = elapsed_per_run(
            || {
                black_box(histogram_parallel(black_box(&input), workers).unwrap());
            },
            iterations,
        );
        println!(
            "workers={workers} parallel={:.3} ms ({:.1} MiB/s) speedup={:.2}x",
            elapsed.as_secs_f64() * 1_000.0,
            byte_count as f64 / elapsed.as_secs_f64() / (1024.0 * 1024.0),
            sequential.as_secs_f64() / elapsed.as_secs_f64()
        );
    }
}
