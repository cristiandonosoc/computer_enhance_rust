use clap::Parser;
use computer_enhance_rust::perf::*;
use winapi::um::memoryapi::{VirtualAlloc, VirtualFree};
use winapi::um::winnt::*;

#[derive(Debug, Parser)]
struct Args {
    pub page_count: usize,

    #[arg(long, default_value = "4096")]
    pub page_size: usize,

    #[arg(long, default_value = "false")]
    pub iterate_backwards: bool,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    unsafe {
        let args = Args::parse();

        let process_handle = open_process()?;

        let buffer_size = args.page_count * args.page_size;

        println!("PAGE_COUNT, TOUCH_COUNT, FAULT_COUNT, EXTRA_FAULTS");

        for touch_count in 0..args.page_count {
            let buffer = VirtualAlloc(
                std::ptr::null_mut(),
                buffer_size,
                MEM_COMMIT | MEM_RESERVE,
                PAGE_READWRITE,
            );
            if buffer.is_null() {
                panic!("VirtualAlloc FAILED");
            }

            // Touch that many pages.
            let page_faults_before = read_page_faults(process_handle);

            for i in 0..touch_count {
                let index = if !args.iterate_backwards {
                    i
                } else {
                    touch_count - 1 - i
                };

                let offset = index * args.page_size + 64;
                let buffer = buffer as *mut u8;
                *buffer.add(offset) = 1;
            }

            let page_faults_after = read_page_faults(process_handle);

            let page_faults = page_faults_after - page_faults_before;

            println!(
                "{}, {}, {}, {}",
                args.page_count,
                touch_count,
                page_faults,
                (page_faults - touch_count as u64)
            );

            if VirtualFree(buffer, 0, MEM_RELEASE) == 0 {
                panic!("VirtualFree FAILED");
            }
        }
    }

    Ok(())
}
