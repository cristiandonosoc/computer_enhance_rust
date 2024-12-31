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

    pub fn run(&mut self, rounds: usize, inverse_order: bool) -> Result<(), Error> {
        let freq = estimate_cpu_frequency();
        let wait = WAIT_SECONDS * freq;

        for round in 0..rounds {
            println!("Run {}", round + 1);
            println!("--------------------------------------------------------------------------");

            for mut i in 0..self.entries.len() {
                if inverse_order {
                    i = self.entries.len() - 1 - i;
                }

                let entry = &mut self.entries[i];

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

                stats.print_stats(freq);
            }
        }

        Ok(())
    }
}

#[derive(Debug)]
struct TestRunStats {
    min: u64,
    max: u64,
    total_cycles: u64,
    count: u64,
    bytes: u64,
}

impl TestRunStats {
    fn new() -> Self {
        Self {
            min: std::u64::MAX,
            max: 0,
            total_cycles: 0,
            count: 0,
            bytes: 0,
        }
    }

    // Returns whether a new minimum was found.
    fn add_run(&mut self, run: &TestRun, freq: u64) -> bool {
        self.count += 1;
        self.bytes = run.bytes;

        let cycles = run.end_timestamp - run.start_timestamp;
        self.total_cycles += cycles;

        if self.max < cycles {
            self.max = cycles;
        }

        if self.min > cycles {
            self.min = cycles;

            println!("New min: {}", print_run(run.bytes, cycles, freq));

            return true;
        }

        return false;
    }

    fn print_stats(&self, freq: u64) {
        println!("- Min: {}", print_run(self.bytes, self.min, freq));
        println!("- Max: {}", print_run(self.bytes, self.max, freq));

        let avg = self.total_cycles / self.count;
        println!("- Avg: {}", print_run(self.bytes, avg, freq));
    }
}

fn print_run(bytes: u64, cycles: u64, freq: u64) -> String {
    let seconds = get_seconds_from_cpu(cycles, freq);
    let bandwidth = (bytes as f64 / seconds) / (GIGABYTE as f64);
    format!("{} ({}) {} GB/s", cycles, print_time(seconds), bandwidth)
}
