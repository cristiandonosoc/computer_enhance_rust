#include "repetition_test.inl"
#include "perf.inl"

#include <assert.h>
#include <memoryapi.h>
#include <windows.h>

constexpr u64 kSize = GIGABYTE;

int main() {

  AddRepetitionTest("WriteAllBytesForward (alloc everytime)", [](TestRun *run) {
    u8 *buffer = (u8 *)VirtualAlloc(NULL, kSize, MEM_COMMIT | MEM_RESERVE,
                                    PAGE_READWRITE);
    assert(buffer);

    run->Bytes = kSize;
    run->StartCycles = ReadCPUTimer();

    // Write all of it.

    for (u64 i = 0; i < kSize; i++) {
      buffer[i] = (u8)i;
    }

    run->EndCycles = ReadCPUTimer();

    VirtualFree(buffer, 0, MEM_RELEASE);
  });

  RunRepetitionTests(10);
}
