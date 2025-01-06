/* Hard-code number of CPUs to simplify demonstration, so we do not have to worry about
 * allocating stack-space at runtime. Could have also reserved stack-space in the linker
 * script but wanted to try to keep it in Rust.
 */
const NCPU: usize = 3;
const STKSZ: usize = 1024 * 64;

/* Set aside statically allocated stack space for each hart.
 * Should be 16 byte aligned as per RISCV calling conventions.
 */
#[repr(align(16), C)]
struct StaticStack([u8; STKSZ * NCPU]);
static mut STACK0: StaticStack = StaticStack([0; STKSZ * NCPU]);

/* This is the location OpenSBI will jump to. OpenSBI can be configured to load the next stage
 * several ways, but the default method in QEMU appears to be the DYNAMIC method. This seems to work
 * by analyzing our ELF passed via -kernel for the address of the beginning of .text. Thus
 * in the linker script we are sure to set the load address to a location that won't overlap
 * the OpenSBI memory (QEMU will complain if we mess up).
 *
 * Initially, only a single boot hart is running, but we also direct OpenSBI to set
 * _start as the entry for other harts when they are woken later by the boot hart in main.
 * Interrupt are also disabled for both machine and supervisor modes.
 *
 * The purpose of this is to simply set up the stack for the hart before calling main.
 *
 * Also when we get here the hart is already running in supervisor mode so the only way
 * to acess machine-mode only features is through SBI calls (shown in main).
 */
#[no_mangle]
#[link_section = ".text"]
pub extern "C" fn _start() {
    /* Don't want to clobber a0 and a1 as they hold hartid and DTB ptr passed by OpenSBI,
     * so use a2 and a3 instead to store temporary values.
     */
    unsafe {
        core::arch::asm!(
        // Set stackpointer to base of STACK0 defined in Rust
        "la sp, {stack0}",

        // Store the stack size in a2
        "li a2, {stksz}",

        // Add one to the current hartid
        "addi a3, a0, 1",

        // Multiply the hartid by the stack size
        "mul a2, a2, a3",

        // Increment the stackpointer by hartid*stksz
        // The stack wil now grow downwards from this point as per convention
        "add sp, sp, a2",

        // Call main defined in Rust
        "call main",

        // Spin in case main ever returns
        "1: j 1b",

        stack0 = sym STACK0,
        stksz = const STKSZ,
        );
    }
}
