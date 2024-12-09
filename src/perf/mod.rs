use core::arch::x86_64;
use std::mem;
use winapi::um::profileapi;

pub fn read_cpu_timer() -> u64 {
    unsafe {
        return x86_64::_rdtsc();
    }
}

pub fn read_os_timer() -> u64 {
    unsafe {
        let mut counter = mem::zeroed();
        profileapi::QueryPerformanceCounter(&mut counter);
        return *counter.QuadPart() as u64;
    }
}

pub fn read_os_freq() -> u64 {
    unsafe {
        let mut freq = mem::zeroed();
        profileapi::QueryPerformanceFrequency(&mut freq);
        return *freq.QuadPart() as u64;
    }
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
