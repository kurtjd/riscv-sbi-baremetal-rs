set confirm off
set architecture riscv:rv64
target remote :1234
symbol-file target/riscv64gc-unknown-none-elf/debug/riscv-sbi-baremetal-rs
set disassemble-next-line auto
set riscv use-compressed-breakpoints yes
