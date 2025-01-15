use std::io::Error;
use winapi::um::processthreadsapi::{GetCurrentProcessId, OpenProcess};
use winapi::um::psapi::*;
use winapi::um::winnt::*;

use super::*;

#[derive(Clone, Debug, Default)]
pub struct TestRun {
    pub start_timestamp: u64,
    pub end_timestamp: u64,
    pub bytes: u64,
    pub start_page_faults: u64,
    pub end_page_faults: u64,
}

impl TestRun {
    fn cycles(&self) -> u64 {
        self.end_timestamp - self.start_timestamp
    }

    fn page_faults(&self) -> u64 {
        self.end_page_faults - self.start_page_faults
    }
}

pub type Handler = Box<dyn FnMut(&mut TestRun) -> Result<(), Error>>;

struct TestEntry {
    name: String,
    handler: Handler,
}

pub struct RepetitionTester {
    entries: Vec<TestEntry>,
    process_handle: HANDLE,
}

const WAIT_SECONDS: u64 = 10;

impl RepetitionTester {
    pub fn new() -> Self {
        let mut tester = Self {
            entries: vec![],
            process_handle: std::ptr::null_mut(),
        };

        unsafe {
            tester.process_handle =
                OpenProcess(PROCESS_QUERY_INFORMATION | PROCESS_VM_READ, 0, GetCurrentProcessId());
        }

        if tester.process_handle.is_null() {
            panic!("null process handle");
        }

        tester
    }

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
                    run.start_page_faults = read_page_faults(self.process_handle);

                    (entry.handler)(&mut run)?;

                    run.end_page_faults = read_page_faults(self.process_handle);

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

    min_run: TestRun,
    max_run: TestRun,
}

impl TestRunStats {
    fn new() -> Self {
        Self {
            min: std::u64::MAX,
            max: 0,
            total_cycles: 0,
            count: 0,
            bytes: 0,
            min_run: TestRun::default(),
            max_run: TestRun::default(),
        }
    }

    // Returns whether a new minimum was found.
    fn add_run(&mut self, run: &TestRun, freq: u64) -> bool {
        self.count += 1;
        self.bytes = run.bytes;

        let cycles = run.cycles();
        self.total_cycles += cycles;

        if self.max < cycles {
            self.max = cycles;
            self.max_run = run.clone();
        }

        if self.min > cycles {
            self.min = cycles;
            self.min_run = run.clone();

            println!("New min: {}", print_run_stats(run.bytes, run.cycles(), freq));

            return true;
        }

        return false;
    }

    fn print_stats(&self, freq: u64) {
        println!("- Min: {}", print_run(&self.min_run, freq));
        println!("- Max: {}", print_run(&self.max_run, freq));

        let avg = self.total_cycles / self.count;
        println!("- Avg: {}", print_run_stats(self.bytes, avg, freq));
    }
}

fn print_run(run: &TestRun, freq: u64) -> String {
    let cycles = run.cycles();
    let seconds = get_seconds_from_cpu(cycles, freq);
    let bandwidth = (run.bytes as f64 / seconds) / (GIGABYTE as f64);

    let mut bytes_per_fault = run.bytes as f64;
    if run.page_faults() > 0 {
        bytes_per_fault = (run.bytes as f64) / (run.page_faults() as f64);
    }

    format!(
        "{} ({}) {} GB/s - Page Faults: {} ({:.4} bytes/fault)",
        cycles,
        print_time(seconds),
        bandwidth,
        run.page_faults(),
        bytes_per_fault,
    )
}

fn print_run_stats(bytes: u64, cycles: u64, freq: u64) -> String {
    let seconds = get_seconds_from_cpu(cycles, freq);
    let bandwidth = (bytes as f64 / seconds) / (GIGABYTE as f64);
    format!("{} ({}) {} GB/s", cycles, print_time(seconds), bandwidth)
}

fn read_page_faults(process_handle: HANDLE) -> u64 {
    unsafe {
        let mut pmc: PROCESS_MEMORY_COUNTERS = std::mem::zeroed();

        if GetProcessMemoryInfo(
            process_handle,
            &mut pmc,
            size_of::<PROCESS_MEMORY_COUNTERS>() as u32,
        ) != 0
        {
            return pmc.PageFaultCount as u64;
        } else {
            panic!("GetProcessMemoryInfo FAILED");
        }
    }
}
