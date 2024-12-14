use super::*;
use num_format::*;

#[macro_export]
macro_rules! profile_block {
    ($label:expr) => {
        crate::perf::profiler::start_entry($label);
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

        crate::perf::profiler::start_entry(type_name_of(__function));
        let mut __profiler_scope = crate::perf::profiler::ProfilerScope::new();
    };
}

#[macro_export]
macro_rules! start_profiling_block {
    ($label:expr) => {{
        crate::perf::profiler::start_entry($label);
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

struct Profiler {
    cpu_freq: u64,
    start_cycles: u64,
    end_cycles: u64,
    total_seconds: f64,

    markers: [u64; PROFILER_ENTRIES],

    stack: [u16; STACK_SIZE],
    stack_top: usize,

    entries: Vec<ProfilerEntry>,
}

static mut PROFILER: Profiler = Profiler {
    cpu_freq: 0,
    start_cycles: 0,
    end_cycles: 0,
    total_seconds: 0.0,

    markers: [0; PROFILER_ENTRIES],

    stack: [0; STACK_SIZE],
    stack_top: 0,

    entries: Vec::new(),
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
        self.entries.reserve(PROFILER_ENTRIES);

        self.start_entry("program");
    }

    fn shutdown(&mut self) {
        self.end_cycles = read_cpu_timer();
        self.end_entry(self.cycles());

        if self.stack_top != 0 {
            panic!("Not all entries were closed!");
        }
    }

    fn running(&self) -> bool {
        self.start_cycles > 0 && self.end_cycles == 0
    }

    fn start_entry(&mut self, label: &'static str) {
        let label_marker = label.as_ptr() as u64;

        // Find if there is an index.
        let mut index: usize = 0;
        for (i, marker) in self.markers.iter().enumerate() {
            if *marker == label_marker {
                index = i;
                break;
            }
        }

        // If we didn't find an entry, we have to add a new one.
        if index == 0 {
            self.markers[self.entries.len()] = label_marker;
            self.entries.push(ProfilerEntry::new(label));
            index = self.entries.len() - 1;
        }

        let entry = &mut self.entries[index];
        entry.call_count += 1;

        // Add it to the stack.
        self.stack[self.stack_top] = index as u16;
        self.stack_top += 1;
    }

    fn end_entry(&mut self, cycles: u64) {
        // Pop the stack.
        let index = self.stack[self.stack_top - 1] as usize;
        self.stack_top -= 1;

        let marker = self.entries[index].marker();

        // Simple case.
        if self.stack_top == 0 {
            self.entries[index].cycles += cycles;
            return;
        }

        //// Add timing to the parent scope *IF* it's not the same.
        //let parent_index = self.stack[self.stack_top - 1] as usize;
        //if marker != self.entries[parent_index].marker() {
        //    self.entries[parent_index].children_cycles += cycles;
        //}

        // Walk the stack to see if this current marker is already there.
        // If the current entry is in the stack already, then we should not add the cycles, because
        // we would double (or more) counting.
        let mut in_stack_already = false;
        for i in 0..self.stack_top {
            let index = self.stack[i] as usize;
            if self.entries[index].marker() == marker {
                in_stack_already = true;
                break;
            }
        }

        if !in_stack_already {
            self.entries[index].cycles += cycles;
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

pub fn start_entry(name: &'static str) {
    unsafe {
        PROFILER.start_entry(name);
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

        println!("{}", PROFILER.entries[0]);
        for (_, entry) in PROFILER.entries.iter().skip(1).enumerate() {
            println!("- {}", entry);
        }
    }
}

#[derive(Clone, Copy, Debug, Default)]
struct ProfilerEntry {
    label: &'static str,
    call_count: u64,
    cycles: u64,
    children_cycles: u64,
}

impl ProfilerEntry {
    fn new(label: &'static str) -> Self {
        ProfilerEntry {
            label,
            ..Default::default()
        }
    }

    fn marker(&self) -> u64 {
        self.label.as_ptr() as u64
    }
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
                    "{}[{}] - Cycles: {} , Time: {:.4}s (Inclusive: {:.4}%, Exclusive: {:.4}%)",
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
