# Computer Enhance Rust

Me doing the [Computer Enhance](https://www.computerenhance.com) course using Rust.

## Disassembler

The first part of the course is a disassembler, which gets old-school x86 binary and generates back
the assembly that represents to it.

The tests currently take a listing (example) assembly, passes it to [Nasm](https://www.nasm.us) to
get the binary and pass it to our disassembler. The test then writes that output, passes it to Nasm
again to verify that the output is the same as the original.


