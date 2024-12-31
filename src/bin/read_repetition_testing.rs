use clap::Parser;
use computer_enhance_rust::{perf::repetition_testing::*, perf::*};
use get_last_error::Win32Error;
use std::ffi::OsStr;
use std::io::{Error, ErrorKind};
use std::iter::once;
use std::os::windows::ffi::OsStrExt;
use std::ptr::null_mut;
use winapi::um::{fileapi::*, handleapi::*, winbase::*, winnt::*};

#[derive(Parser)]
struct Args {
    pub input: String,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    let mut tester = RepetitionTester::default();

    {
        let input = args.input.clone();
        let filename: Vec<u16> = OsStr::new(args.input.as_str())
            .encode_wide()
            .chain(once(0))
            .collect();

        tester.add_test(
            "Windows API".to_string(),
            Box::new(move |run| {
                let size: usize = std::fs::metadata(&input)?.len() as usize;

                unsafe {
                    let mut buffer: Vec<u8> = vec![];
                    buffer.reserve(size);

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
                        return Err(Error::new(ErrorKind::Other, "CreateFileW"));
                    }

                    SetFilePointer(handle, 0, null_mut(), FILE_BEGIN);

                    run.start_timestamp = read_cpu_timer();

                    let mut bytes_read: u32 = 0;
                    if ReadFile(
                        handle,
                        buffer.as_mut_ptr() as _,
                        size as u32,
                        &mut bytes_read,
                        null_mut(),
                    ) == 0
                    {
                        CloseHandle(handle);

                        let error = Win32Error::get_last_error();
                        let message = format!("ReadFile: {}", error);
                        return Err(Error::new(ErrorKind::Other, message));
                    }

                    run.end_timestamp = read_cpu_timer();
                    run.bytes += bytes_read as u64;

                    CloseHandle(handle);
                }

                Ok(())
            }),
        );
    }

    {
        let input = args.input.clone();
        tester.add_test(
            "Read".to_string(),
            Box::new(move |run| {
                run.start_timestamp = read_cpu_timer();

                let bytes = std::fs::read(&input)?;

                run.end_timestamp = read_cpu_timer();
                run.bytes = bytes.len() as u64;

                Ok(())
            }),
        );
    }

    tester.run(10)?;

    Ok(())
}
