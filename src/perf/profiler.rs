use super::*;
use num_format::*;
use prettytable::Table;

#[macro_export]
macro_rules! profile_block {
    ($label:expr) => {
        unsafe {
            static INIT: std::sync::Once = std::sync::Once::new();
            static mut INDEX: u16 = 0;
            INIT.call_once(|| {
                INDEX = crate::perf::profiler::get_next_index();
            });
            crate::perf::profiler::start_entry(INDEX, $label);
        }

        let mut __profiler_scope = crate::perf::profiler::ProfilerScope::new();
    };
}

#[macro_export]
macro_rules! profile_function {
    () => {
        fn __function() {}
        fn type_name_of<T>(_: T) -> &'static str {
            std::any::type_name::<T>()
        }

        unsafe {
            static INIT: std::sync::Once = std::sync::Once::new();
            static mut INDEX: u16 = 0;
            INIT.call_once(|| {
                INDEX = crate::perf::profiler::get_next_index();
            });

            crate::perf::profiler::start_entry(INDEX, type_name_of(__function));
        }
        let mut __profiler_scope = crate::perf::profiler::ProfilerScope::new();
    };
}

#[macro_export]
macro_rules! start_profiling_block {
    ($label:expr) => {{
        unsafe {
            static INIT: std::sync::Once = std::sync::Once::new();
            static mut INDEX: u16 = 0;
            INIT.call_once(|| {
                INDEX = crate::perf::profiler::get_next_index();
            });

            crate::perf::profiler::start_entry(INDEX, $label);
        }

        crate::perf::read_cpu_timer()
    }};
}

#[macro_export]
macro_rules! end_profiling_block {
    ($start:ident) => {{
        let end_cycles = crate::perf::read_cpu_timer();
        crate::perf::profiler::end_entry(end_cycles - $start);
    }};
}

pub struct ProfilerScope {
    start_cycles: u64,
}

impl ProfilerScope {
    pub fn new() -> Self {
        Self {
            start_cycles: read_cpu_timer(),
        }
    }
}

impl Drop for ProfilerScope {
    fn drop(&mut self) {
        let end_cycles = read_cpu_timer();
        end_entry(end_cycles - self.start_cycles);
    }
}

const PROFILER_ENTRIES: usize = 4096;
const STACK_SIZE: usize = 128;

const DEFAULT_ENTRY: ProfilerEntry = ProfilerEntry {
    label: "",
    call_count: 0,
    cycles: 0,
    children_cycles: 0,
    ref_count: 0,
};

struct Profiler {
    cpu_freq: u64,
    start_cycles: u64,
    end_cycles: u64,
    total_seconds: f64,

    stack: [u16; STACK_SIZE],
    stack_top: usize,

    entries: [ProfilerEntry; PROFILER_ENTRIES],
    next_entry_index: u16,
}

static mut PROFILER: Profiler = Profiler {
    cpu_freq: 0,
    start_cycles: 0,
    end_cycles: 0,
    total_seconds: 0.0,

    stack: [0; STACK_SIZE],
    stack_top: 0,

    entries: [DEFAULT_ENTRY; PROFILER_ENTRIES],
    next_entry_index: 0,
};

impl Profiler {
    fn init(&mut self) {
        if self.running() {
            panic!("init called more than once!");
        }

        if self.stack_top != 0 {
            panic!("Entries already started before calling init to profiler! Did you call |init_profiler|?");
        }

        self.cpu_freq = estimate_cpu_frequency();
        self.start_cycles = read_cpu_timer();
        self.next_entry_index = 1;

        self.start_entry(0, "program");
    }

    fn shutdown(&mut self) {
        self.end_cycles = read_cpu_timer();
        self.end_entry(self.cycles());

        for i in 0..self.next_entry_index {
            let entry = &self.entries[i as usize];
            if entry.ref_count > 0 {
                panic!(
                    "entry \"{}\" is still active (ref_count: {})",
                    entry.label, entry.ref_count
                );
            }
        }

        if self.stack_top != 0 {
            panic!("Not all entries were closed!");
        }
    }

    fn running(&self) -> bool {
        self.start_cycles > 0 && self.end_cycles == 0
    }

    fn start_entry(&mut self, index: u16, label: &'static str) {
        let entry = &mut self.entries[index as usize];
        entry.label = label;
        entry.call_count += 1;
        entry.ref_count += 1;

        // Add it to the stack.
        self.stack[self.stack_top] = index;
        self.stack_top += 1;
    }

    fn end_entry(&mut self, cycles: u64) {
        // Pop the stack.
        let index = self.stack[self.stack_top - 1] as usize;
        self.stack_top -= 1;

        // If the ref-count is more than one, it means that this entry was twice in the stack, so
        // it would be double counting.
        self.entries[index].ref_count -= 1;
        if self.entries[index].ref_count == 0 {
            self.entries[index].cycles += cycles;
        }

        // Add timing to the parent scope if it's not in the stack already.
        // Otherwise it's double counting again.
        if self.stack_top > 0 {
            let parent_index = self.stack[self.stack_top - 1] as usize;
            if self.entries[parent_index].ref_count == 1 {
                self.entries[parent_index].children_cycles += cycles;
            }
        }
    }

    fn cycles(&self) -> u64 {
        self.end_cycles - self.start_cycles
    }
}

pub fn init_profiler() {
    unsafe {
        PROFILER.init();
    }
}

pub fn shutdown_profiler() {
    unsafe {
        PROFILER.shutdown();
    }
}

pub fn get_next_index() -> u16 {
    unsafe {
        let index = PROFILER.next_entry_index;
        PROFILER.next_entry_index += 1;
        return index;
    }
}

pub fn start_entry(index: u16, label: &'static str) {
    unsafe {
        PROFILER.start_entry(index, label);
    }
}

pub fn end_entry(cycles: u64) {
    unsafe {
        PROFILER.end_entry(cycles);
    }
}

pub fn print_timings() {
    unsafe {
        if PROFILER.running() {
            println!("Cannot print_timings. Profiler still running. Call shutdown_profiler first");
            return;
        }

        if PROFILER.stack_top != 0 {
            panic!("Stack did not finish correctly");
        }

        let freq = PROFILER.cpu_freq;
        PROFILER.total_seconds = get_seconds_from_cpu(PROFILER.cycles(), freq);

        let mut table = Table::new();
        table.add_row(row![
            "NAME",
            "TIME",
            "CALL COUNT",
            "CYCLES",
            "AVG. CYCLES/CALL",
            "AVG. TIME/CALL",
            "INCLUSIVE",
            "EXCLUSIVE"
        ]);

        for i in 0..PROFILER.next_entry_index {
            let entry = &mut PROFILER.entries[i as usize];
            add_timing_row(&mut table, entry);
        }

        table.printstd();
    }
}

fn add_timing_row(table: &mut Table, entry: &ProfilerEntry) {
    unsafe {
        let label = strip_function_suffix(entry.label);
        let section_seconds = get_seconds_from_cpu(entry.cycles, PROFILER.cpu_freq);

        let exclusive_cycles = entry.cycles - entry.children_cycles;

        let inclusive = 100.0 * (entry.cycles as f64) / (PROFILER.cycles() as f64);
        let exclusive = 100.0 * (exclusive_cycles as f64) / (PROFILER.cycles() as f64);

        let avg_cycles = (entry.cycles as f64) / (entry.call_count as f64);
        let avg_time = (section_seconds as f64) / (entry.call_count as f64);

        table.add_row(row![
            label,
            print_time(section_seconds),
            entry.call_count.to_formatted_string(&Locale::en),
            entry.cycles.to_formatted_string(&Locale::en),
            format!("{:.4}", avg_cycles),
            print_time(avg_time),
            format!("{:.4}%", inclusive),
            format!("{:.4}%", exclusive),
        ]);
    }
}

#[derive(Clone, Copy, Debug, Default)]
struct ProfilerEntry {
    label: &'static str,
    call_count: u64,
    cycles: u64,
    children_cycles: u64,
    ref_count: u16,
}

impl std::fmt::Display for ProfilerEntry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        unsafe {
            let label = strip_function_suffix(self.label);
            let section_seconds = get_seconds_from_cpu(self.cycles, PROFILER.cpu_freq);

            if self.children_cycles > 0 {
                let exclusive_cycles = self.cycles - self.children_cycles;

                write!(
                    f,
                    "{}[{}] - Cycles: {} | Time: {:.4}s (Inclusive: {:.4}%, Exclusive: {:.4}%)",
                    label,
                    self.call_count.to_formatted_string(&Locale::en),
                    self.cycles.to_formatted_string(&Locale::en),
                    section_seconds,
                    100.0 * (self.cycles as f64) / (PROFILER.cycles() as f64),
                    100.0 * (exclusive_cycles as f64) / (PROFILER.cycles() as f64),
                )
            } else {
                write!(
                    f,
                    "{}[{}] - Cycles: {} , Time: {:.4}s ({:.4}%)",
                    label,
                    self.call_count.to_formatted_string(&Locale::en),
                    self.cycles.to_formatted_string(&Locale::en),
                    section_seconds,
                    100.0 * (self.cycles as f64) / (PROFILER.cycles() as f64),
                )
            }
        }
    }
}

fn strip_function_suffix(s: &str) -> &str {
    if s.ends_with("::__function") {
        &s[..s.len() - 12]
    } else {
        s
    }
}
