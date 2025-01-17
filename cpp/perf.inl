#pragma once

#include <Windows.h>
#include <format>
#include <intrin.h>
#include <profileapi.h>

using u8 = uint8_t;
using u64 = uint64_t;
using f64 = double;

// Cycles
// ------------------------------------------------------------------------------------------

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

// Time
// --------------------------------------------------------------------------------------------

f64 CPUCyclesToSeconds(u64 cycles, u64 freq) {
  return static_cast<f64>(cycles) / static_cast<f64>(freq);
}

constexpr f64 MICROSECOND = 0.000001;
constexpr f64 MILLISECOND = 0.001;
constexpr f64 SECOND = 1.0;
constexpr f64 MINUTE = 60.0;
constexpr f64 HOUR = 60.0 * MINUTE;
constexpr f64 DAY = 24.0 * HOUR;

std::string PrintTime(f64 seconds) {
  if (seconds > DAY) {
    return std::format("{:.4} days", seconds / DAY);
  } else if (seconds > HOUR) {
    return std::format("{:.4} hours", seconds / HOUR);
  } else if (seconds > MINUTE) {
    return std::format("{:.4} m", seconds / MINUTE);
  } else if (seconds > SECOND) {
    return std::format("{:.4} s", seconds);
  } else if (seconds > MILLISECOND) {
    return std::format("{:.4} ms", seconds * 1000.0);
  } else if (seconds > MICROSECOND) {
    return std::format("{:.4} us", seconds * 1'000'000.0);
  } else {
    return std::format("{:.4} ns", seconds * 1'000'000'000.0);
  }
}

// Memory
// ------------------------------------------------------------------------------------------

constexpr u64 KILOBYTE = 1024;
constexpr u64 MEGABYTE = 1024 * 1024;
constexpr u64 GIGABYTE = 1024 * 1024 * 1024;

std::string PrintBytes(u64 bytes) {
  if (bytes < KILOBYTE) {
    return std::format("{}", bytes);
  } else if (bytes < MEGABYTE) {
    f64 fbytes = static_cast<f64>(bytes) / static_cast<f64>(KILOBYTE);
    return std::format("{:.4} KB", fbytes);
  } else if (bytes < GIGABYTE) {
    f64 fbytes = static_cast<f64>(bytes) / static_cast<f64>(MEGABYTE);
    return std::format("{:.4} MB", fbytes);
  } else {
    f64 fbytes = static_cast<f64>(bytes) / static_cast<f64>(GIGABYTE);
    return std::format("{:.4} GB", fbytes);
  }
}
