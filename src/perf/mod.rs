pub mod profiler;
pub mod repetition_testing;

use core::arch::x86_64;
use get_last_error::Win32Error;
use std::io::{Error, ErrorKind};
use winapi::um::memoryapi::{VirtualAlloc, VirtualFree};
use winapi::um::processthreadsapi::{GetCurrentProcessId, OpenProcess};
use winapi::um::profileapi;
use winapi::um::psapi::*;
use winapi::um::winnt::*;

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

pub const DAY: f64 = 24.0 * HOUR;
pub const HOUR: f64 = 60.0 * MINUTE;
pub const MINUTE: f64 = 60.0;
pub const SECOND: f64 = 1.0;
pub const MILLISECOND: f64 = 0.001;
pub const MICROSECOND: f64 = 0.000_001;

pub fn print_time(seconds: f64) -> String {
    if seconds > DAY {
        return format!("{:.4} days", seconds / DAY);
    } else if seconds > HOUR {
        return format!("{:.4} hours", seconds / HOUR);
    } else if seconds > MINUTE {
        return format!("{:.4} m", seconds / MINUTE);
    } else if seconds > SECOND {
        return format!("{:.4} s", seconds);
    } else if seconds > MILLISECOND {
        return format!("{:.4} ms", seconds * 1000.0);
    } else if seconds > MICROSECOND {
        return format!("{:.4} us", seconds * 1_000_000.0);
    } else {
        return format!("{:.4} ns", seconds * 1_000_000_000.0);
    }
}

pub const KILOBYTE: u64 = 1024;
pub const MEGABYTE: u64 = 1024 * 1024;
pub const GIGABYTE: u64 = 1024 * 1024 * 1024;

pub fn print_bytes(bytes: u64) -> String {
    if bytes < KILOBYTE {
        return format!("{}", bytes);
    } else if bytes < MEGABYTE {
        let fbytes = (bytes as f64) / (KILOBYTE as f64);
        return format!("{:.4} KB", fbytes);
    } else if bytes < GIGABYTE {
        let fbytes = (bytes as f64) / (MEGABYTE as f64);
        return format!("{:.4} MB", fbytes);
    } else {
        let fbytes = (bytes as f64) / (GIGABYTE as f64);
        return format!("{:.4} GB", fbytes);
    }
}

pub fn open_process() -> Result<HANDLE, std::io::Error> {
    unsafe {
        let handle =
            OpenProcess(PROCESS_QUERY_INFORMATION | PROCESS_VM_READ, 0, GetCurrentProcessId());
        if handle.is_null() {
            let error = Win32Error::get_last_error();
            let message = format!("OpenProcess: {}", error);
            return Err(Error::new(ErrorKind::Other, message));
        }

        Ok(handle)
    }
}

pub fn read_page_faults(process_handle: HANDLE) -> u64 {
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

pub fn virtual_alloc(size: usize) -> *mut u8 {
    unsafe {
        let buffer =
            VirtualAlloc(std::ptr::null_mut(), size, MEM_COMMIT | MEM_RESERVE, PAGE_READWRITE)
                as *mut u8;
        if buffer.is_null() {
            panic!("VirtualAlloc FAILED");
        }

        buffer
    }
}

pub fn virtual_free(buffer: *mut u8) {
    unsafe {
        if VirtualFree(buffer as _, 0, MEM_RELEASE) == 0 {
            panic!("VirtualFree FAILED");
        }
    }
}
