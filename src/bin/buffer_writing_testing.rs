use clap::Parser;
use computer_enhance_rust::{perf::repetition_testing::*, perf::*};

#[derive(Clone, Debug, Parser)]
struct Args {
    // 256 * 1024
    #[arg(long, default_value = "262144")]
    pub page_count: u64,

    #[arg(long, default_value = "4096")]
    pub page_size: u64,
}

impl Args {
    fn total_size(&self) -> u64 {
        self.page_count * self.page_size
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    println!("TOTAL SIZE: {}", print_bytes(args.total_size()));

    let mut tester = RepetitionTester::new();

    let mut buffers = vec![];

    buffers.push(add_write_test("Forward (alloc everytime)", &args, &mut tester, true, true));
    buffers.push(add_write_test("Backward (alloc everytime)", &args, &mut tester, true, false));

    buffers.push(add_write_test("Forward (alloc once)", &args, &mut tester, false, true));
    buffers.push(add_write_test("Backward (alloc once)", &args, &mut tester, false, false));

    tester.run(10, false)?;

    for opt_buffer in buffers {
        if let Some(buffer) = opt_buffer {
            virtual_free(buffer);
        }
    }

    Ok(())
}

fn add_write_test(
    name: &str,
    args: &Args,
    tester: &mut RepetitionTester,
    alloc_everytime: bool,
    forward: bool,
) -> Option<*mut u8> {
    unsafe {
        let total_size = args.total_size();
        let mut buffer: *mut u8 = std::ptr::null_mut();

        if !alloc_everytime {
            buffer = virtual_alloc(total_size as usize);
        }

        let process_handle = tester.process_handle.clone();

        tester.add_test(
            name.to_string(),
            Box::new(move |run| {
                if alloc_everytime {
                    buffer = virtual_alloc(total_size as usize);
                }

                run.start_timestamp = read_cpu_timer();
                run.start_page_faults = read_page_faults(process_handle);

                if forward {
                    for i in 0..total_size {
                        *buffer.wrapping_add(i as usize) = 1;
                        // let pf = read_page_faults(process_handle) - run.start_page_faults;
                        // println!("{}: PF: {}", i, pf);
                        // avg_page += pf;
                        // println!("{}: PF: {}", i, pf - run.start_page_faults);
                    }
                } else {
                    for i in (0..total_size).rev() {
                        *buffer.wrapping_add(i as usize) = 1;
                        // let pf = read_page_faults(process_handle) - run.start_page_faults;
                        // println!("{}: PF: {}", total_size -1 - i, pf);
                        // let pf = read_page_faults(process_handle) - run.start_page_faults;
                        // avg_page += pf;
                    }
                }

                run.end_timestamp = read_cpu_timer();
                run.end_page_faults = read_page_faults(process_handle);
                run.bytes += total_size;

                if alloc_everytime {
                    virtual_free(buffer);
                }

                Ok(())
            }),
        );

        if !alloc_everytime {
            return Some(buffer);
        }

        None
    }
}
