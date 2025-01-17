#include "repetition_test.inl"

#include <assert.h>
#include <memoryapi.h>
#include <windows.h>

#include "perf.inl"

constexpr u64 kSize = GIGABYTE;

u8* gBuffer = nullptr;

void AddWriteRepetitionTest(const char* name, bool alloc_everytime,
                            bool forward) {
    auto test = [alloc_everytime, forward](TestRun* run) {
        u8* buffer = gBuffer;
        if (alloc_everytime) {
            buffer = static_cast<u8*>(VirtualAlloc(
                NULL, kSize, MEM_COMMIT | MEM_RESERVE, PAGE_READWRITE));
        }
        assert(buffer);

        run->Bytes = kSize;
        run->StartCycles = ReadCPUTimer();
        run->StartPageFaults = ReadPageFaults();

        // Write all of it.
        if (forward) {
            for (u64 i = 0; i < kSize; i++) {
                buffer[i] = (u8)i;
            }
        } else {
            for (i64 i = kSize - 1; i >= 0; i--) {
                buffer[i] = (u8)i;
            }
        }

        run->EndCycles = ReadCPUTimer();
        run->EndPageFaults = ReadPageFaults();

        if (alloc_everytime) {
            VirtualFree(buffer, 0, MEM_RELEASE);
        }
    };

    AddRepetitionTest(name, std::move(test));
}

int main() {
    InitProcessData();

    gBuffer = (u8*)VirtualAlloc(NULL, kSize, MEM_COMMIT | MEM_RESERVE,
                                PAGE_READWRITE);

    AddWriteRepetitionTest("Forward (alloc everytime)", true, true);
    AddWriteRepetitionTest("Backward (alloc everytime)", true, false);

    AddWriteRepetitionTest("Forward (alloc once)", false, true);
    AddWriteRepetitionTest("Backward (alloc once)", false, false);

    RunRepetitionTests(10);

    VirtualFree(gBuffer, 0, MEM_RELEASE);
}
