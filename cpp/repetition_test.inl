#pragma once

#include "perf.inl"

#include <functional>

struct TestRun {
  u64 Bytes = 0;

  u64 StartCycles = 0;
  u64 EndCycles = 0;

  u64 StartPageFaults = 0;
  u64 EndPageFaults = 0;
};

std::string PrintTestRun(const TestRun &run, u64 freq) {
  u64 cycles = run.EndCycles - run.StartCycles;
  f64 seconds = CPUCyclesToSeconds(cycles, freq);

  f64 bandwidth = static_cast<f64>(run.Bytes) / seconds;
  bandwidth /= static_cast<f64>(GIGABYTE);

  f64 bytes_per_page_fault = static_cast<f64>(run.Bytes);
  u64 page_faults = run.EndPageFaults - run.StartPageFaults;
  if (page_faults > 0) {
    bytes_per_page_fault /= static_cast<f64>(page_faults);
  }

  return std::format("{} ({}) {} GB/s - Page Faults: {} ({:.4} bytes/fault)",
                     cycles, PrintTime(seconds), bandwidth, page_faults,
                     bytes_per_page_fault);
}

struct RepetitionTest {
  using HandlerFunc = std::function<void(TestRun *)>;

  const char *Name = nullptr;
  HandlerFunc Handler;
  std::vector<TestRun> Runs;
};

namespace {
std::vector<RepetitionTest> gTests;
} // namespace

void AddRepetitionTest(const char *name, RepetitionTest::HandlerFunc handler) {
  gTests.push_back({
      .Name = name,
      .Handler = std::move(handler),
  });
}

void RunRepetitionTests(u64 rounds) {
  u64 freq = EstimateCPUFrequency();

  while (true) {
    for (auto &test : gTests) {
      printf("Running Test: %s\n", test.Name);
      for (u64 i = 0; i < rounds; i++) {

        TestRun run = {};
        test.Handler(&run);

        std::string print = PrintTestRun(run, freq);
        printf("- %s\n", print.c_str());

        test.Runs.push_back(std::move(run));
      }
    }
  }
}
