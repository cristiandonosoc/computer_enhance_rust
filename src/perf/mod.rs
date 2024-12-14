pub mod profiler;

use core::arch::x86_64;
use winapi::um::profileapi;

#[inline]
pub fn read_cpu_timer() -> u64 {
    unsafe {
        return x86_64::_rdtsc();
    }
}

pub fn read_os_timer() -> u64 {
    unsafe {
        #[allow(invalid_value)]
        let mut counter = std::mem::MaybeUninit::uninit().assume_init();
        profileapi::QueryPerformanceCounter(&mut counter);
        return *counter.QuadPart() as u64;
    }
}

pub fn read_os_freq() -> u64 {
    unsafe {
        #[allow(invalid_value)]
        let mut freq = std::mem::MaybeUninit::uninit().assume_init();
        profileapi::QueryPerformanceFrequency(&mut freq);
        return *freq.QuadPart() as u64;
    }
}

pub fn estimate_cpu_frequency() -> u64 {
    estimate_cpu_frequency_detailed(100)
}

pub fn estimate_cpu_frequency_detailed(wait_time: u64) -> u64 {
    let cpu_start = read_cpu_timer();
    let os_start = read_os_timer();

    let mut os_end: u64;
    let mut os_elapsed: u64 = 0;

    let os_freq = read_os_freq();
    let os_wait_time = os_freq * wait_time / 1000;
    while os_elapsed < os_wait_time {
        os_end = read_os_timer();
        os_elapsed = os_end - os_start;
    }

    let cpu_end = read_cpu_timer();
    let cpu_elapsed = cpu_end - cpu_start;

    let mut cpu_freq: u64 = 0;
    if os_elapsed > 0 {
        cpu_freq = os_freq * cpu_elapsed / os_elapsed;
    }

    cpu_freq
}

pub fn get_seconds_from_cpu(cycles: u64, freq: u64) -> f64 {
    (cycles as f64) / (freq as f64)
}

pub fn print_freq(freq: u64) -> String {
    let mut freq = freq as f64;
    if freq < 1000.0 {
        return format!("{} Hz", freq);
    }

    freq = freq / 1000.0;
    if freq < 1000.0 {
        return format!("{} KHz", freq);
    }

    freq = freq / 1000.0;
    if freq < 1000.0 {
        return format!("{} MHz", freq);
    }

    freq = freq / 1000.0;
    return format!("{} GHz", freq);
}
