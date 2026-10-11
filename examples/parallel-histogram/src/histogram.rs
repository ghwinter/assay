use std::num::NonZeroUsize;

pub const BUCKET_COUNT: usize = 256;

pub type Histogram = [usize; BUCKET_COUNT];

#[derive(Debug, PartialEq, Eq)]
pub enum HistogramError {
    ZeroWorkers,
}

pub fn histogram_sequential(input: &[u8]) -> Histogram {
    let mut counts = [0; BUCKET_COUNT];
    for byte in input {
        counts[*byte as usize] += 1;
    }
    counts
}

pub fn histogram_parallel(input: &[u8], workers: usize) -> Result<Histogram, HistogramError> {
    let requested_workers = NonZeroUsize::new(workers).ok_or(HistogramError::ZeroWorkers)?;
    if input.is_empty() {
        return Ok([0; BUCKET_COUNT]);
    }
    let worker_count = requested_workers.get().min(input.len());
    if worker_count == 1 {
        return Ok(histogram_sequential(input));
    }
    let chunk_size = input.len().div_ceil(worker_count);
    Ok(std::thread::scope(|scope| {
        let handles: Vec<_> = input
            .chunks(chunk_size)
            .map(|chunk| scope.spawn(move || histogram_sequential(chunk)))
            .collect();
        let mut total = [0; BUCKET_COUNT];
        for handle in handles {
            let partial = handle.join().expect("histogram worker panicked");
            for (total_count, partial_count) in total.iter_mut().zip(partial) {
                *total_count += partial_count;
            }
        }
        total
    }))
}

#[cfg(test)]
mod tests {
    use super::{histogram_parallel, histogram_sequential, HistogramError, BUCKET_COUNT};

    #[test]
    fn empty_input_has_zero_counts_for_any_positive_worker_count() {
        for workers in [1, 2, 7, usize::MAX] {
            assert_eq!(histogram_parallel(&[], workers), Ok([0; BUCKET_COUNT]));
        }
    }

    #[test]
    fn rejects_zero_workers_even_when_input_is_empty() {
        assert_eq!(histogram_parallel(&[], 0), Err(HistogramError::ZeroWorkers));
        assert_eq!(
            histogram_parallel(&[1], 0),
            Err(HistogramError::ZeroWorkers)
        );
    }

    #[test]
    fn counts_every_byte_value() {
        let input: Vec<u8> = (u8::MIN..=u8::MAX).collect();
        assert_eq!(histogram_sequential(&input), [1; BUCKET_COUNT]);
        for workers in [1, 2, 3, 8, 256, 512] {
            assert_eq!(histogram_parallel(&input, workers), Ok([1; BUCKET_COUNT]));
        }
    }

    #[test]
    fn parallel_result_matches_reference_across_chunk_boundaries() {
        for length in [1, 2, 3, 7, 8, 9, 31, 32, 33, 255, 256, 257, 1025] {
            let input: Vec<u8> = (0..length)
                .map(|index| ((index * 73 + index / 7) % BUCKET_COUNT) as u8)
                .collect();
            let expected = histogram_sequential(&input);
            for workers in [1, 2, 3, 4, 7, 16, length + 3] {
                let actual = histogram_parallel(&input, workers).unwrap();
                assert_eq!(actual, expected, "length={length}, workers={workers}");
                assert_eq!(actual.iter().sum::<usize>(), input.len());
            }
        }
    }
}
