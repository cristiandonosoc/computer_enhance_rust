use clap::Parser;
use computer_enhance_rust::{args, perf};
use log::info;

#[derive(Parser)]
struct Args {
    #[arg(long, default_value = "1000")]
    pub milliseconds_to_wait: u64,

    #[command(flatten)]
    base: args::BaseArgs,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    computer_enhance_rust::args::evaluate_log(&args.base);

    let os_freq = perf::read_os_freq();
    info!("OS Frequency (reported): {} ({})", os_freq, perf::print_freq(os_freq));

    let cpu_start = perf::read_cpu_timer();
    let os_start = perf::read_os_timer();

    let mut os_end: u64 = 0;
    let mut os_elapsed: u64 = 0;

    let os_wait_time = os_freq * args.milliseconds_to_wait / 1000;
    while os_elapsed < os_wait_time {
        os_end = perf::read_os_timer();
        os_elapsed = os_end - os_start;
    }

    let cpu_end = perf::read_cpu_timer();
    let cpu_elapsed = cpu_end - cpu_start;

    let mut cpu_freq: u64 = 0;
    if os_elapsed > 0 {
        cpu_freq = os_freq * cpu_elapsed / os_elapsed;
    }

    info!("OS Timer: {} -> {} = {} elapsed", os_start, os_end, os_elapsed);
    info!("OS Seconds: {}", (os_elapsed as f64) / (os_freq as f64));

    info!("CPU Timer: {} -> {} = {} elapsed", cpu_start, cpu_end, cpu_elapsed);
    info!("CPU Freq (guessed): {} ({})", cpu_freq, perf::print_freq(cpu_freq));

    Ok(())
}
