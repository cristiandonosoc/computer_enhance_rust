use clap::Parser;
use computer_enhance_rust::defer;
use computer_enhance_rust::perf::*;
use get_last_error::Win32Error;
use std::ffi::OsStr;
use std::io::{Error, ErrorKind};
use std::iter::once;
use std::os::windows::ffi::OsStrExt;
use std::ptr::null_mut;
use winapi::um::{fileapi::*, handleapi::*, winbase::*, winnt::*};

#[derive(Debug, Parser)]
struct Args {
    pub input: String,

    #[arg(long, default_value = "false")]
    pub alloc_everytime: bool,

    #[arg(long, default_value = "50")]
    pub alternate_count: usize,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    let size: usize = std::fs::metadata(&args.input)?.len() as usize;

    let filename: Vec<u16> = OsStr::new(args.input.as_str())
        .encode_wide()
        .chain(once(0))
        .collect();

    let freq = estimate_cpu_frequency();

    unsafe {
        let global_buffer: *mut u8;
        let mut run_buffer: *mut u8 = null_mut();

        global_buffer = libc::malloc(size) as _;
        defer!({
            libc::free(global_buffer as _);
        });

        const LOOP_COUNT: usize = 1000000;

        for loopi in 0..LOOP_COUNT {
            let is_even_loop = loopi & 1 == 0;
            let do_alloc = args.alloc_everytime || !is_even_loop;

            println!("Making {} runs, alloc every run: {}", args.alternate_count, do_alloc);
            for _ in 0..args.alternate_count {
                let handle = CreateFileW(
                    filename.as_ptr(),
                    GENERIC_READ,
                    FILE_SHARE_READ | FILE_SHARE_WRITE,
                    null_mut(),
                    OPEN_EXISTING,
                    FILE_ATTRIBUTE_NORMAL,
                    null_mut(),
                );

                if handle.is_null() {
                    return Err(Box::new(Error::new(ErrorKind::Other, "CreateFileW")));
                }

                SetFilePointer(handle, 0, null_mut(), FILE_BEGIN);

                if do_alloc {
                    run_buffer = libc::malloc(size) as _;
                }

                defer!({
                    if do_alloc {
                        CloseHandle(handle);
                        libc::free(run_buffer as _);
                    }
                });

                let start_timestamp = read_cpu_timer();

                let buffer = if !do_alloc { global_buffer } else { run_buffer };

                let mut bytes_read: u32 = 0;
                if ReadFile(handle, buffer as _, size as u32, &mut bytes_read, null_mut()) == 0 {
                    let error = Win32Error::get_last_error();
                    let message = format!("ReadFile: {}", error);
                    return Err(Box::new(Error::new(ErrorKind::Other, message)));
                }

                let end_timestamp = read_cpu_timer();
                let bytes = bytes_read as u64;

                let cycles = end_timestamp - start_timestamp;
                let seconds = get_seconds_from_cpu(cycles, freq);
                let bandwidth = (bytes as f64 / seconds) / (GIGABYTE as f64);

                println!("Read: {} ({}) {} GB/s", cycles, print_time(seconds), bandwidth)
            }
        }
    }

    Ok(())
}
