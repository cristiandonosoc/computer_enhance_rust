#include <Windows.h>
#include <format>
#include <intrin.h>
#include <profileapi.h>

using u64 = uint64_t;
using f64 = double;

u64 ReadCPUTimer() { return __rdtsc(); }

u64 ReadOSTimer() {
  LARGE_INTEGER counter;
  QueryPerformanceCounter(&counter);
  return counter.QuadPart;
}

u64 ReadOSFreq() {
  LARGE_INTEGER freq;
  QueryPerformanceFrequency(&freq);
  return freq.QuadPart;
}

u64 EstimateCPUFrequency(u64 wait_time_ms = 100) {
  u64 cpu_start = ReadCPUTimer();
  u64 os_start = ReadOSTimer();

  u64 os_end = 0;
  u64 os_elapsed = 0;

  u64 os_freq = ReadOSFreq();
  u64 os_wait_time = os_freq * wait_time_ms / 1000;
  while (os_elapsed < os_wait_time) {
    os_end = ReadOSTimer();
    os_elapsed = os_end - os_start;
  }

  u64 cpu_end = ReadCPUTimer();
  u64 cpu_elapsed = cpu_end - cpu_start;

  u64 cpu_freq = 0;
  if (os_elapsed > 0) {
    cpu_freq = os_freq * cpu_elapsed / os_elapsed;
  }

  return cpu_freq;
}

std::string PrintFreq(u64 frequency) {
  f64 freq = static_cast<f64>(frequency);
  if (freq < 1000.0) {
    return std::format("{:.4} Hz", freq);
  }

  freq = freq / 1000.0;
  if (freq < 1000.0) {
    return std::format("{:.4} KHz", freq);
  }

  freq = freq / 1000.0;
  if (freq < 1000.0) {
    return std::format("{:.4} MHz", freq);
  }

  freq = freq / 1000.0;
  return std::format("{:.4} GHz", freq);
}
