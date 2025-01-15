use clap::Parser;
use computer_enhance_rust::{perf::repetition_testing::*, perf::*};
use get_last_error::Win32Error;
use libc;
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
    pub inverse_order: bool,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    let mut tester = RepetitionTester::default();

    add_fs_read_test(&args, &mut tester);
    add_windows_api_alloc_everytime_test(&args, &mut tester);
    let buffer = add_windows_api_alloc_once_test(&args, &mut tester);

    tester.run(10, args.inverse_order)?;

    unsafe {
        libc::free(buffer as _);
    }
    Ok(())
}

fn add_fs_read_test(args: &Args, tester: &mut RepetitionTester) {
    let input = args.input.clone();
    tester.add_test(
        "std::fs::read".to_string(),
        Box::new(move |run| {
            run.start_timestamp = read_cpu_timer();

            let bytes = std::fs::read(&input)?;

            run.end_timestamp = read_cpu_timer();
            run.bytes = bytes.len() as u64;

            Ok(())
        }),
    );
}

fn add_windows_api_alloc_everytime_test(args: &Args, tester: &mut RepetitionTester) {
    let input = args.input.clone();
    let filename: Vec<u16> = OsStr::new(args.input.as_str())
        .encode_wide()
        .chain(once(0))
        .collect();

    tester.add_test(
        "Windows ReadFile (alloc everytime)".to_string(),
        Box::new(move |run| {
            let size: usize = std::fs::metadata(&input)?.len() as usize;

            unsafe {
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

                let buffer: *mut u8 = libc::malloc(size) as _;

                run.start_timestamp = read_cpu_timer();

                let mut bytes_read: u32 = 0;
                if ReadFile(
                    handle,
                    //buffer.as_mut_ptr() as _,
                    buffer as _,
                    size as u32,
                    &mut bytes_read,
                    null_mut(),
                ) == 0
                {
                    CloseHandle(handle);
                    libc::free(buffer as _);

                    let error = Win32Error::get_last_error();
                    let message = format!("ReadFile: {}", error);
                    return Err(Error::new(ErrorKind::Other, message));
                }

                run.end_timestamp = read_cpu_timer();
                run.bytes += bytes_read as u64;

                CloseHandle(handle);
                libc::free(buffer as _);
            }

            Ok(())
        }),
    );
}

fn add_windows_api_alloc_once_test(args: &Args, tester: &mut RepetitionTester) -> *mut u8 {
    let input = args.input.clone();
    let filename: Vec<u16> = OsStr::new(args.input.as_str())
        .encode_wide()
        .chain(once(0))
        .collect();

    let filesize: usize;
    let buffer: *mut u8;

    unsafe {
        filesize = std::fs::metadata(&input).unwrap().len() as usize;
        buffer = libc::malloc(filesize) as _;
    }

    tester.add_test(
        "Windows ReadFile (alloc once)".to_string(),
        Box::new(move |run| {
            unsafe {
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
                if ReadFile(handle, buffer as _, filesize as u32, &mut bytes_read, null_mut()) == 0
                {
                    CloseHandle(handle);
                    libc::free(buffer as _);

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

    buffer
}
