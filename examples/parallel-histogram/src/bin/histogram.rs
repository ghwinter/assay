use parallel_histogram::{process_reader, PipelineConfig};
use std::fs::File;
use std::io::{self, BufReader};
use std::process::ExitCode;

const DEFAULT_CHUNK_BYTES: usize = 64 * 1024;

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("error: {error}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let mut arguments = std::env::args().skip(1);
    let Some(input_path) = arguments.next() else {
        return Err(usage_error().into());
    };
    if input_path == "--help" || input_path == "-h" {
        println!("{}", usage());
        return Ok(());
    }

    let mut workers = std::thread::available_parallelism()
        .map(usize::from)
        .unwrap_or(1);
    let mut chunk_bytes = DEFAULT_CHUNK_BYTES;
    let mut queue_capacity = workers.saturating_mul(2).max(1);
    while let Some(option) = arguments.next() {
        let value = arguments.next().ok_or_else(usage_error)?;
        let parsed = value.parse::<usize>().map_err(|_| usage_error())?;
        match option.as_str() {
            "--workers" => workers = parsed,
            "--chunk-bytes" => chunk_bytes = parsed,
            "--queue-capacity" => queue_capacity = parsed,
            _ => return Err(usage_error().into()),
        }
    }

    let config = PipelineConfig::new(workers, chunk_bytes, queue_capacity)?;
    let input = BufReader::new(File::open(input_path)?);
    let summary = process_reader(input, config)?;
    let buckets = summary
        .histogram
        .iter()
        .map(usize::to_string)
        .collect::<Vec<_>>()
        .join(",");
    println!(
        "{{\"bytes_read\":{},\"chunks\":{},\"workers\":{},\"histogram\":[{}]}}",
        summary.bytes_read,
        summary.chunks_processed,
        config.workers(),
        buckets
    );
    Ok(())
}

fn usage() -> &'static str {
    "Usage: histogram <input-file> [--workers N] [--chunk-bytes N] [--queue-capacity N]"
}

fn usage_error() -> io::Error {
    io::Error::new(io::ErrorKind::InvalidInput, usage())
}
