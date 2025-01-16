#include <iostream>
#include <intrin.h>
#include <Windows.h>

using u64 = uint64_t;

u64 ReadCPUTimer() {
	return __rdtsc();
}

u64 ReadOSTimer() {
	LARGE_INTEGER counter;
	QueryPerformanceCounter(&counter);
	return counter.QuadPart;
}

void main() {
	std::cout << "RDTSC: " << ReadCPUTimer() << std::endl;
	std::cout << "QueryPerformanceCounter: " << ReadOSTimer() << std::endl;
}

