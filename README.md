# RISC-V SBI Baremetal in Rust
This repo demonstrates how a SMP RV64 baremetal program written in Rust, which is booted via OpenSBI, can make SBI calls and access the device tree.

Essentially, this is my understanding so far of the minimum needed to get everything setup correctly using Rust with minimal assembly. I've commented the code thoroughly to explain what is happening and my thought process.

Feel free to submit PRs if something doesn't seem right or could be done better!

## Run
Ensure `qemu-system-riscv64` is installed on your system then run: `cargo run`  

This will start a qemu instance with the default bios (OpenSBI) and load this binary as the kernel (which OpenSBI jumps to).

## Debug
Ensure `qemu-system-riscv64` and gdb are installed on your system.

In one terminal run:
`cargo gdb`  

This will start a qemu instance as above but pause execution to wait for commands from gdb. Then in another terminal run `gdb` (or possibly `gdb-multiarch` depending on your system).

## License
This project is licensed under the MIT license and is completely free to use and modify.