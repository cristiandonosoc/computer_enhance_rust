#include "repetition_test.inl"
#include "perf.inl"

#include <assert.h>
#include <memoryapi.h>
#include <windows.h>

constexpr u64 kSize = GIGABYTE;

int main() {
  InitProcessData();

  AddRepetitionTest("WriteAllBytesForward (alloc everytime)", [](TestRun *run) {
    u8 *buffer = (u8 *)VirtualAlloc(NULL, kSize, MEM_COMMIT | MEM_RESERVE,
                                    PAGE_READWRITE);
    assert(buffer);

    run->Bytes = kSize;
    run->StartCycles = ReadCPUTimer();
    run->StartPageFaults = ReadPageFaults();

    // Write all of it.

    for (u64 i = 0; i < kSize; i++) {
      buffer[i] = (u8)i;
    }

    run->EndCycles = ReadCPUTimer();
    run->EndPageFaults = ReadPageFaults();

    VirtualFree(buffer, 0, MEM_RELEASE);
  });

  RunRepetitionTests(10);
}
