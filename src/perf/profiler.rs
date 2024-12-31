use super::*;
use num_format::*;
use prettytable::Table;

#[macro_export]
#[cfg(feature = "profile")]
macro_rules! profile_block {
    ($label:expr, $bytes:expr) => {
        let __profiler_scope_index: u16;
        unsafe {
            static INIT: std::sync::Once = std::sync::Once::new();
            static mut INDEX: u16 = 0;
            INIT.call_once(|| {
                INDEX = crate::perf::profiler::get_next_index();
            });
            __profiler_scope_index = INDEX;
        }
        let __profiler_scope =
            crate::perf::profiler::start_entry(__profiler_scope_index, $label, $bytes as u64);
    };

    ($label:expr) => {
        $crate::profile_block!($label, 0);
    };
}

#[macro_export]
#[cfg(feature = "profile")]
macro_rules! profile_function {
    () => {
        $crate::profile_function!(0);
    };

    ($bytes:expr) => {
        fn __function() {}
        fn type_name_of<T>(_: T) -> &'static str {
            std::any::type_name::<T>()
        }

        $crate::profile_block!(type_name_of(__function), $bytes as u64);
    };
}

#[macro_export]
#[cfg(not(feature = "profile"))]
macro_rules! profile_block {
    ($label:expr, $bytes:expr) => {};
    ($label:expr) => {};
}

#[macro_export]
#[cfg(not(feature = "profile"))]
macro_rules! profile_function {
    () => {};
    ($bytes:expr) => {};
}

pub struct ProfilerScope {
    start_cycles: u64,
    old_inclusive: u64,
}

impl Drop for ProfilerScope {
    fn drop(&mut self) {
        end_entry(&self);
    }
}

const PROFILER_ENTRIES: usize = 16;
const STACK_SIZE: usize = 128;

struct Profiler {
    cpu_freq: u64,
    start_cycles: u64,
    end_cycles: u64,
    total_seconds: f64,

    entries: [ProfilerEntry; PROFILER_ENTRIES],
    stack: [u16; STACK_SIZE],

    stack_top: u16,
    next_entry_index: u16,
}

static mut PROFILER: Profiler = Profiler {
    cpu_freq: 0,
    start_cycles: 0,
    end_cycles: 0,
    total_seconds: 0.0,

    entries: [DEFAULT_PROFILER_ENTRY; PROFILER_ENTRIES],
    stack: [0; STACK_SIZE],

    stack_top: 0,
    next_entry_index: 0,
};

#[derive(Clone, Copy, Debug, Default)]
struct ProfilerEntry {
    label: &'static str,
    call_count: u64,
    cycles_exclusive: u64,
    cycles_inclusive: u64,
    bytes: u64,
}

const DEFAULT_PROFILER_ENTRY: ProfilerEntry = ProfilerEntry {
    label: "",
    call_count: 0,
    cycles_exclusive: 0,
    cycles_inclusive: 0,
    bytes: 0,
};

impl Profiler {
    fn running(&self) -> bool {
        self.start_cycles > 0 && self.end_cycles == 0
    }

    fn cycles(&self) -> u64 {
        self.end_cycles - self.start_cycles
    }

    fn entry_count(&self) -> usize {
        return self.next_entry_index as usize;
    }
}

pub fn init_profiler() {
    unsafe {
        if PROFILER.running() {
            panic!("init called more than once!");
        }

        // Entry 0 is the program.
        PROFILER.next_entry_index = 1;
        PROFILER.stack_top = 1;

        PROFILER.cpu_freq = estimate_cpu_frequency();
        PROFILER.start_cycles = read_cpu_timer();
    }
}

pub fn shutdown_profiler() {
    unsafe {
        PROFILER.end_cycles = read_cpu_timer();

        // Massage the "program" entry.
        let entry = &mut PROFILER.entries[0];
        entry.label = "Program";
        entry.call_count = 1;
        let elapsed = PROFILER.end_cycles - PROFILER.start_cycles;
        entry.cycles_inclusive = elapsed;
        entry.cycles_exclusive = u64::wrapping_add(entry.cycles_exclusive, elapsed);
    }
}

// 0 is unused.
pub fn get_next_index() -> u16 {
    unsafe {
        let index = PROFILER.next_entry_index;
        PROFILER.next_entry_index += 1;
        return index;
    }
}

pub fn start_entry(index: u16, label: &'static str, bytes: u64) -> ProfilerScope {
    unsafe {
        let entry = &mut PROFILER.entries[index as usize];
        entry.label = label;
        entry.call_count = u64::wrapping_add(entry.call_count, 1);
        entry.bytes = u64::wrapping_add(entry.bytes, bytes);

        // Push to stack.
        PROFILER.stack[PROFILER.stack_top as usize] = index;
        PROFILER.stack_top = u16::wrapping_add(PROFILER.stack_top, 1);

        return ProfilerScope {
            start_cycles: read_cpu_timer(),
            old_inclusive: entry.cycles_inclusive,
        };
    }
}

pub fn end_entry(scope: &ProfilerScope) {
    unsafe {
        let end_cycles = read_cpu_timer();
        let elapsed = u64::wrapping_sub(end_cycles, scope.start_cycles);

        // Pop from the stack.
        PROFILER.stack_top = u16::wrapping_sub(PROFILER.stack_top, 1);
        let index = PROFILER.stack[PROFILER.stack_top as usize] as usize;

        let parent_stack_top = u16::wrapping_sub(PROFILER.stack_top, 1);
        let parent_index = PROFILER.stack[parent_stack_top as usize] as usize;

        PROFILER.entries[parent_index].cycles_exclusive =
            u64::wrapping_sub(PROFILER.entries[parent_index].cycles_exclusive, elapsed);
        PROFILER.entries[index].cycles_exclusive =
            u64::wrapping_add(PROFILER.entries[index].cycles_exclusive, elapsed);

        // For the inclusive, we track the time of this scope.
        // If it's recursive, the outer most will update the value in the end anyway.
        PROFILER.entries[index].cycles_inclusive = u64::wrapping_add(scope.old_inclusive, elapsed);
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

        let mut table = Table::new();
        table.add_row(row![
            "NAME",
            "CALL COUNT",
            "TIME",
            "AVG. CYCLES/CALL",
            "AVG. TIME/CALL",
            "CYCLES INCLUSIVE",
            "CYCLES EXCLUSIVE",
            "BYTES PROCESSED",
            "BANDWIDTH",
        ]);

        for i in 0..PROFILER.entry_count() {
            let entry = &mut PROFILER.entries[i as usize];
            add_timing_row(&mut table, entry);
        }

        table.printstd();
    }
}

fn add_timing_row(table: &mut Table, entry: &ProfilerEntry) {
    unsafe {
        let locale = &Locale::en;

        let label = strip_function_suffix(entry.label);
        let section_seconds = get_seconds_from_cpu(entry.cycles_inclusive, PROFILER.cpu_freq);

        let inclusive_str = entry.cycles_inclusive.to_formatted_string(locale);
        let inclusive_pct = 100.0 * (entry.cycles_inclusive as f64) / (PROFILER.cycles() as f64);

        let exclusive_str = entry.cycles_exclusive.to_formatted_string(locale);
        let exclusive_pct = 100.0 * (entry.cycles_exclusive as f64) / (PROFILER.cycles() as f64);

        let avg_cycles = (entry.cycles_inclusive as f64) / (entry.call_count as f64);
        let avg_time = (section_seconds as f64) / (entry.call_count as f64);

        let mut bandwidth = (entry.bytes as f64) / (section_seconds as f64);
        bandwidth /= GIGABYTE as f64;

        table.add_row(row![
            label,
            entry.call_count.to_formatted_string(locale),
            print_time(section_seconds),
            format!("{:.4}", avg_cycles),
            print_time(avg_time),
            format!("{} ({:.4}%)", inclusive_str, inclusive_pct),
            format!("{} ({:.4}%)", exclusive_str, exclusive_pct),
            print_bytes(entry.bytes),
            format!("{:.4} GB/s", bandwidth),
        ]);
    }
}

fn strip_function_suffix(s: &str) -> &str {
    if s.ends_with("::__function") {
        &s[..s.len() - 12]
    } else {
        s
    }
}
