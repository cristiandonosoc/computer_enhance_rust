#include "perf.inl"

#include <iostream>

int main() {
  std::cout << "RDTSC: " << ReadCPUTimer() << std::endl;
  std::cout << "QueryPerformanceCounter: " << ReadOSTimer() << std::endl;

  u64 freq = EstimateCPUFrequency();
  std::cout << "Estimated CPU Frequency: " << freq << ", " << PrintFreq(freq)
            << std::endl;
}
