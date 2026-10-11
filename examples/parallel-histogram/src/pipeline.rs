use crate::{Histogram, BUCKET_COUNT};
use std::io::{self, Read};
use std::num::NonZeroUsize;
use std::sync::{mpsc, Arc, Mutex};
use std::thread;

#[derive(Debug, PartialEq, Eq)]
pub enum ConfigError {
    ZeroWorkers,
    ZeroChunkBytes,
    ZeroQueueCapacity,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PipelineConfig {
    workers: NonZeroUsize,
    chunk_bytes: NonZeroUsize,
    queue_capacity: NonZeroUsize,
}

impl PipelineConfig {
    pub fn new(
        workers: usize,
        chunk_bytes: usize,
        queue_capacity: usize,
    ) -> Result<Self, ConfigError> {
        Ok(Self {
            workers: NonZeroUsize::new(workers).ok_or(ConfigError::ZeroWorkers)?,
            chunk_bytes: NonZeroUsize::new(chunk_bytes).ok_or(ConfigError::ZeroChunkBytes)?,
            queue_capacity: NonZeroUsize::new(queue_capacity)
                .ok_or(ConfigError::ZeroQueueCapacity)?,
        })
    }

    pub fn workers(self) -> usize {
        self.workers.get()
    }

    pub fn chunk_bytes(self) -> usize {
        self.chunk_bytes.get()
    }

    pub fn queue_capacity(self) -> usize {
        self.queue_capacity.get()
    }
}

#[derive(Debug)]
pub enum PipelineError {
    Io(io::Error),
    WorkerPanicked,
    WorkerPoolClosed,
    CountOverflow,
    AccountingMismatch,
}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let message = match self {
            Self::ZeroWorkers => "worker count must be positive",
            Self::ZeroChunkBytes => "chunk size must be positive",
            Self::ZeroQueueCapacity => "queue capacity must be positive",
        };
        formatter.write_str(message)
    }
}

impl std::error::Error for ConfigError {}

impl std::fmt::Display for PipelineError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let message = match self {
            Self::Io(error) => return write!(formatter, "input read failed: {error}"),
            Self::WorkerPanicked => "a histogram worker panicked",
            Self::WorkerPoolClosed => "all histogram workers exited before input completed",
            Self::CountOverflow => "input or histogram count exceeds the supported range",
            Self::AccountingMismatch => "worker accounting does not match submitted input",
        };
        formatter.write_str(message)
    }
}

impl std::error::Error for PipelineError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io(error) => Some(error),
            _ => None,
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct ProcessSummary {
    pub histogram: Histogram,
    pub bytes_read: usize,
    pub chunks_submitted: usize,
    pub chunks_processed: usize,
}

struct Chunk {
    bytes: Vec<u8>,
}

struct WorkerReport {
    histogram: Histogram,
    bytes_processed: usize,
    chunks_processed: usize,
}

pub fn process_reader<R: Read>(
    mut reader: R,
    config: PipelineConfig,
) -> Result<ProcessSummary, PipelineError> {
    let (sender, receiver) = mpsc::sync_channel::<Chunk>(config.queue_capacity());
    let receiver = Arc::new(Mutex::new(receiver));

    let (read_result, worker_result) = thread::scope(|scope| {
        let handles: Vec<_> = (0..config.workers())
            .map(|_| {
                let receiver = Arc::clone(&receiver);
                scope.spawn(move || process_chunks(receiver))
            })
            .collect();

        let read_result = read_chunks(&mut reader, &sender, config.chunk_bytes());
        drop(sender);

        let mut aggregate = WorkerReport {
            histogram: [0; BUCKET_COUNT],
            bytes_processed: 0,
            chunks_processed: 0,
        };
        let mut first_error = None;
        for handle in handles {
            match handle.join() {
                Ok(Ok(report)) => {
                    if merge_report(&mut aggregate, report).is_err() {
                        first_error.get_or_insert(PipelineError::CountOverflow);
                    }
                }
                Ok(Err(error)) => {
                    first_error.get_or_insert(error);
                }
                Err(_) => {
                    first_error.get_or_insert(PipelineError::WorkerPanicked);
                }
            }
        }
        (read_result, first_error.map_or(Ok(aggregate), Err))
    });

    let (bytes_read, chunks_submitted) = read_result?;
    let aggregate = worker_result?;
    let counted_bytes = aggregate
        .histogram
        .iter()
        .try_fold(0usize, |total, count| total.checked_add(*count))
        .ok_or(PipelineError::CountOverflow)?;
    if aggregate.bytes_processed != bytes_read
        || aggregate.chunks_processed != chunks_submitted
        || counted_bytes != bytes_read
    {
        return Err(PipelineError::AccountingMismatch);
    }

    Ok(ProcessSummary {
        histogram: aggregate.histogram,
        bytes_read,
        chunks_submitted,
        chunks_processed: aggregate.chunks_processed,
    })
}

fn read_chunks<R: Read>(
    reader: &mut R,
    sender: &mpsc::SyncSender<Chunk>,
    chunk_bytes: usize,
) -> Result<(usize, usize), PipelineError> {
    let mut buffer = vec![0; chunk_bytes];
    let mut bytes_read = 0usize;
    let mut chunks_submitted = 0usize;
    loop {
        let length = match reader.read(&mut buffer) {
            Ok(0) => break,
            Ok(length) => length,
            Err(error) if error.kind() == io::ErrorKind::Interrupted => continue,
            Err(error) => return Err(PipelineError::Io(error)),
        };
        bytes_read = bytes_read
            .checked_add(length)
            .ok_or(PipelineError::CountOverflow)?;
        sender
            .send(Chunk {
                bytes: buffer[..length].to_vec(),
            })
            .map_err(|_| PipelineError::WorkerPoolClosed)?;
        chunks_submitted = chunks_submitted
            .checked_add(1)
            .ok_or(PipelineError::CountOverflow)?;
    }
    Ok((bytes_read, chunks_submitted))
}

fn process_chunks(
    receiver: Arc<Mutex<mpsc::Receiver<Chunk>>>,
) -> Result<WorkerReport, PipelineError> {
    let mut report = WorkerReport {
        histogram: [0; BUCKET_COUNT],
        bytes_processed: 0,
        chunks_processed: 0,
    };
    loop {
        let next_chunk = {
            let guard = receiver.lock().map_err(|_| PipelineError::WorkerPanicked)?;
            guard.recv()
        };
        let chunk = match next_chunk {
            Ok(chunk) => chunk,
            Err(_) => break,
        };
        report.bytes_processed = report
            .bytes_processed
            .checked_add(chunk.bytes.len())
            .ok_or(PipelineError::CountOverflow)?;
        report.chunks_processed = report
            .chunks_processed
            .checked_add(1)
            .ok_or(PipelineError::CountOverflow)?;
        for byte in chunk.bytes {
            let count = &mut report.histogram[byte as usize];
            *count = count.checked_add(1).ok_or(PipelineError::CountOverflow)?;
        }
    }
    Ok(report)
}

fn merge_report(target: &mut WorkerReport, partial: WorkerReport) -> Result<(), PipelineError> {
    target.bytes_processed = target
        .bytes_processed
        .checked_add(partial.bytes_processed)
        .ok_or(PipelineError::CountOverflow)?;
    target.chunks_processed = target
        .chunks_processed
        .checked_add(partial.chunks_processed)
        .ok_or(PipelineError::CountOverflow)?;
    for (total, count) in target.histogram.iter_mut().zip(partial.histogram) {
        *total = total
            .checked_add(count)
            .ok_or(PipelineError::CountOverflow)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{process_reader, ConfigError, PipelineConfig, PipelineError};
    use crate::histogram_sequential;
    use std::io::{self, Cursor, Read};

    struct ShortReader<'a> {
        bytes: &'a [u8],
        max_read: usize,
    }

    impl Read for ShortReader<'_> {
        fn read(&mut self, output: &mut [u8]) -> io::Result<usize> {
            let count = output.len().min(self.max_read).min(self.bytes.len());
            output[..count].copy_from_slice(&self.bytes[..count]);
            self.bytes = &self.bytes[count..];
            Ok(count)
        }
    }

    struct FailingReader {
        remaining: Cursor<Vec<u8>>,
        fail_after: usize,
        bytes_read: usize,
    }

    impl Read for FailingReader {
        fn read(&mut self, output: &mut [u8]) -> io::Result<usize> {
            if self.bytes_read >= self.fail_after {
                return Err(io::Error::other("injected read failure"));
            }
            let allowed = output.len().min(self.fail_after - self.bytes_read);
            let count = self.remaining.read(&mut output[..allowed])?;
            self.bytes_read += count;
            Ok(count)
        }
    }

    fn config(workers: usize, chunk_bytes: usize, queue_capacity: usize) -> PipelineConfig {
        PipelineConfig::new(workers, chunk_bytes, queue_capacity).unwrap()
    }

    #[test]
    fn rejects_zero_configuration_values() {
        assert_eq!(PipelineConfig::new(0, 4, 1), Err(ConfigError::ZeroWorkers));
        assert_eq!(
            PipelineConfig::new(2, 0, 1),
            Err(ConfigError::ZeroChunkBytes)
        );
        assert_eq!(
            PipelineConfig::new(2, 4, 0),
            Err(ConfigError::ZeroQueueCapacity)
        );
    }

    #[test]
    fn bounded_pipeline_matches_reference_and_accounts_for_every_chunk() {
        for length in [0, 1, 3, 4, 5, 31, 257, 4097] {
            let input: Vec<u8> = (0..length)
                .map(|index| ((index * 83 + index / 11) % 256) as u8)
                .collect();
            for workers in [1, 2, 4, 9] {
                let summary = process_reader(Cursor::new(&input), config(workers, 7, 1)).unwrap();
                assert_eq!(summary.histogram, histogram_sequential(&input));
                assert_eq!(summary.bytes_read, input.len());
                assert_eq!(summary.chunks_submitted, summary.chunks_processed);
                assert_eq!(summary.histogram.iter().sum::<usize>(), summary.bytes_read);
            }
        }
    }

    #[test]
    fn short_reads_are_combined_into_the_exact_result() {
        let input: Vec<u8> = (0..127).map(|index| (index * 29 % 256) as u8).collect();
        let reader = ShortReader {
            bytes: &input,
            max_read: 2,
        };
        let summary = process_reader(reader, config(3, 16, 1)).unwrap();
        assert_eq!(summary.histogram, histogram_sequential(&input));
        assert_eq!(summary.bytes_read, input.len());
    }

    #[test]
    fn read_failure_returns_error_after_worker_shutdown() {
        let reader = FailingReader {
            remaining: Cursor::new(vec![17; 64]),
            fail_after: 21,
            bytes_read: 0,
        };
        let result = process_reader(reader, config(4, 5, 1));
        assert!(matches!(result, Err(PipelineError::Io(_))));
    }
}
