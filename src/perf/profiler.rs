use super::*;
use std::collections::HashMap;
use treeline::Tree;
use num_format::*;

#[macro_export]
macro_rules! profile_block {
    ($msg:expr) => {
        crate::perf::profiler::start_profiling_block($msg);
        let mut __profiler_scope = crate::perf::profiler::ProfilerScope {};
    };
}

#[macro_export]
macro_rules! profile_function {
    () => {
        fn __function() {}
        fn type_name_of<T>(_: T) -> &'static str {
            std::any::type_name::<T>()
        }

        crate::perf::profiler::start_profiling_block(type_name_of(__function));
        let mut __profiler_scope = crate::perf::profiler::ProfilerScope {};
    };
}

pub struct ProfilerScope {}

impl Drop for ProfilerScope {
    fn drop(&mut self) {
        end_profiling_block();
    }
}

struct Profiler {
    cpu_freq: u64,
    start_cycles: u64,
    end_cycles: u64,
    total_seconds: f64,

    current_entry_index: i32,
    entries: Vec<ProfilerEntry>,
}

impl Profiler {
    fn init(&mut self) {
        if self.running() {
            panic!("init called more than once!");
        }

        if self.current_entry_index != INDEX_NONE {
            panic!("Entries already started before calling init to profiler! Did you call |init_profiler|?");
        }

        self.cpu_freq = estimate_cpu_frequency();
        self.start_cycles = read_cpu_timer();
        self.entries.reserve(128);

        self.start_entry("program");
    }

    fn shutdown(&mut self) {
        self.end_entry();

        if self.current_entry_index != INDEX_NONE {
            panic!("Not all entries were closed!");
        }
        self.end_cycles = read_cpu_timer();
    }

    fn running(&self) -> bool {
        self.start_cycles > 0 && self.end_cycles == 0
    }

    fn start_entry(&mut self, name: &'static str) {
        let entry = ProfilerEntry::new(name, self.current_entry_index);
        self.entries.push(entry);
        self.current_entry_index = (self.entries.len() as i32) - 1;
    }

    fn end_entry(&mut self) {
        let entry = &mut self.entries[self.current_entry_index as usize];
        entry.end();
        self.current_entry_index = entry.parent_index;
    }

    fn cycles(&self) -> u64 {
        self.end_cycles - self.start_cycles
    }
}

static INDEX_NONE: i32 = -1;

static mut PROFILER: Profiler = Profiler {
    cpu_freq: 0,
    start_cycles: 0,
    end_cycles: 0,
    total_seconds: 0.0,
    current_entry_index: INDEX_NONE,
    entries: Vec::<ProfilerEntry>::new(),
};

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

pub fn start_profiling_block(name: &'static str) {
    unsafe {
        PROFILER.start_entry(name);
    }
}

pub fn end_profiling_block() {
    unsafe {
        PROFILER.end_entry();
    }
}

pub fn print_timings() {
    unsafe {
        if PROFILER.running() {
            println!("Cannot print_timings. Profiler still running. Call shutdown_profiler first");
            return;
        }

        let freq = PROFILER.cpu_freq;
        PROFILER.total_seconds = get_seconds_from_cpu(PROFILER.cycles(), freq);

        // Collect all the children for each entry.
        let mut children_map: HashMap<i32, Vec<i32>> = HashMap::new();
        for (i, entry) in PROFILER.entries.iter().enumerate() {
            if entry.parent_index != INDEX_NONE {
                children_map
                    .entry(entry.parent_index)
                    .or_insert(Vec::new())
                    .push(i as i32);
            }
        }

        let mut tree = Tree::<String>::root(format!(
            "Total Time: {:.4}s - CPU freq. {} (~{})",
            PROFILER.total_seconds,
            freq.to_formatted_string(&Locale::en),
            print_freq(freq)
        ));


        add_children(&mut tree, &children_map, 0);

        println!("{}", tree);
    }
}

fn add_children(tree: &mut Tree<String>, children_map: &HashMap<i32, Vec<i32>>, index: i32) {
    unsafe {
        let mut child_indexes: Vec<i32> = children_map.get(&index).unwrap_or(&Vec::new()).clone();
        child_indexes.sort();

        for child_index in child_indexes {
            let child_entry = &PROFILER.entries[child_index as usize];
            let mut child_tree = Tree::<String>::root(child_entry.to_string());
            add_children(&mut child_tree, children_map, child_index);
            tree.push(child_tree);
        }
    }
}

#[derive(Debug, Clone)]
struct ProfilerEntry {
    name: &'static str,
    start_cycles: u64,
    end_cycles: u64,
    parent_index: i32,
}

impl ProfilerEntry {
    fn new(name: &'static str, parent_index: i32) -> Self {
        ProfilerEntry {
            name,
            start_cycles: read_cpu_timer(),
            end_cycles: 0,
            parent_index,
        }
    }

    fn end(&mut self) {
        self.end_cycles = read_cpu_timer();
    }

    fn cycles(&self) -> u64 {
        self.end_cycles - self.start_cycles
    }
}

impl std::fmt::Display for ProfilerEntry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        unsafe {
            let section_cycles = self.cycles();
            let section_seconds = get_seconds_from_cpu(section_cycles, PROFILER.cpu_freq);

            let name = strip_function_suffix(self.name);

            write!(
                f,
                "{} - Cycles: {}, Time: {:.4}s ({:.4}%)",
                name,
                section_cycles.to_formatted_string(&Locale::en),
                section_seconds,
                100.0 * (section_cycles as f64) / (PROFILER.cycles() as f64)
            )
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
