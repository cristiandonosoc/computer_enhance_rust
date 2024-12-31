use std::io::Error;

use super::*;

#[derive(Debug, Default)]
pub struct TestRun {
    pub start_timestamp: u64,
    pub end_timestamp: u64,
    pub bytes: u64,
}

pub type Handler = Box<dyn FnMut(&mut TestRun) -> Result<(), Error>>;

struct TestEntry {
    name: String,
    handler: Handler,
}

#[derive(Default)]
pub struct RepetitionTester {
    entries: Vec<TestEntry>,
}

const WAIT_SECONDS: u64 = 10;

impl RepetitionTester {
    pub fn add_test(&mut self, name: String, handler: Handler) {
        let entry = TestEntry { name, handler };

        self.entries.push(entry);
    }

    pub fn run(&mut self, rounds: usize) -> Result<(), Error> {
        let freq = estimate_cpu_frequency();
        let wait = WAIT_SECONDS * freq;

        for i in 0..rounds {
            println!("Run {}", i + 1);
            println!("--------------------------------------------------------------------------");

            for entry in &mut self.entries {
                println!("\nTesting {}", entry.name);

                let mut stats = TestRunStats::new();

                let mut best_timestamp = read_cpu_timer();

                loop {
                    // Check if the timer for wait for a better result has been found.
                    let now = read_cpu_timer();
                    if best_timestamp + wait < now {
                        break;
                    }

                    // Do a new run.
                    let mut run = TestRun::default();
                    (entry.handler)(&mut run)?;

                    if stats.add_run(&run, freq) {
                        best_timestamp = now;
                    }
                }
            }
        }

        Ok(())
    }
}

#[derive(Debug)]
struct TestRunStats {
    min: u64,
    max: u64,
    count: u64,
}

impl TestRunStats {
    fn new() -> Self {
        Self {
            min: std::u64::MAX,
            max: 0,
            count: 0,
        }
    }

    // Returns whether a new minimum was found.
    fn add_run(&mut self, run: &TestRun, freq: u64) -> bool {
        self.count += 1;

        let cycles = run.end_timestamp - run.start_timestamp;

        if self.max < cycles {
            self.max = cycles;
        }

        if self.min > cycles {
            self.min = cycles;

            let seconds = get_seconds_from_cpu(cycles, freq);
            let bandwidth = (run.bytes as f64 / seconds) / (GIGABYTE as f64);
            println!("New min: {} ({}) {} GB/s", cycles, print_time(seconds), bandwidth);

            return true;
        }

        return false;
    }
}
