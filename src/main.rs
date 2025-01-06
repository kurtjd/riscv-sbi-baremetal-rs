#![no_std]
#![no_main]

mod start;
use start::_start;

use core::fmt::Write;
use core::panic::PanicInfo;
use core::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use fdt::Fdt;
use heapless::String;
use sbi_rt::Physical;
use spin::Once;

/* Since we know only one hart is executing at startup and we only write to these once
 * from the boot hart, there is no risk to writing to these, but use atomics and spin-locked
 * lazy intialization to satisfy compiler and reduce unsafe usage.
 *
 * Also just use SeqCst ordering in atomics access for simplicity sake.
 */
static DEVTREE: Once<Fdt> = Once::new();
static STARTED: AtomicBool = AtomicBool::new(false);
static NCPU: AtomicUsize = AtomicUsize::new(0);

/* Formats a string and passes the physical address of that string to an SBI call
 * for printing to a debug console. In QEMUs case, this would be the memory-mapped UART.
 *
 * Useful to be able to get debugging output immediately without having to first parse a
 * device tree.
 */
macro_rules! debug_print {
    ($($args:tt)*) => {{
        let mut message: String<0xFF> = String::new();
        let _ = write!(&mut message, $($args)*);

        let sz = message.len();
        let addr = message.as_ptr() as usize;
        let ptr: Physical<&[u8]> = Physical::new(sz, addr, 0);

        /* This can fail, but not a whole lot we can do if can't write to console.
         *
         * Wrapper around:
         *
         * li a7, {EID=0x4442434E}
         * li a6, {FID=0x0}
         * li a0, {num_bytes=sz}
         * li a1, {base_addr_lo=addr}
         * li a2, {base_addr_hi=0}
         * ecall
         */
        sbi_rt::console_write(ptr);
    }}
}

/* Uses the dtb pointer provided by OpenSBI to locate and parse device tree.
 * The handy fdt crate does the actual hard-work of parsing the binary blob located
 * there.
 */
fn handle_dtb(dtb: *const u8) {
    let dt =
        DEVTREE.call_once(|| unsafe { Fdt::from_ptr(dtb).expect("Unable to parse device tree") });

    // Get some useful info from the DT
    let model = dt.root().model();
    let ncpus = dt.cpus().count();
    let mem = dt
        .memory()
        .regions()
        .next()
        .map(|dram| dram.starting_address)
        .expect("Unable to locate DRAM start");

    debug_print!("Device tree info:\n");
    debug_print!("Model: {}\n", model);
    debug_print!("No. CPUs: {}\n", ncpus);
    debug_print!("DRAM start: {:p}\n", mem);

    // Store number of CPUs for later
    NCPU.store(ncpus, Ordering::SeqCst);
}

fn start_harts(boothartid: usize) {
    for h in 0..NCPU.load(Ordering::SeqCst) {
        /* Start the given hartid (will fail if already started).
         *
         * May actually need a better way to get all hartids since I believe
         * hartids don't necessarily need to correspond to the number of harts
         * (except there should always be a hart with hartid 0).
         *
         * We also pass the address of the entry point we wish harts to resume at,
         * and thus we want them to start at _start as well to initialize their stacks.
         *
         * The "opaque" argument gets passed to a1. This is expected to be the pointer
         * to the dtb from main code, but we don't want to access it if not boot hart
         * so just pass 0 as a fail-safe to force panic if hart tries to deref it.
         *
         * Not sure if this is the "proper" way to wake harts but could not find much else.
         *
         * Wrapper around:
         *
         * li a7, {EID=0x4853}
         * li a6, {FID=0x0}
         * li a0, {hartid=h}
         * li a1, {start_addr=_start}
         * li a2, {opaque=0}
         * ecall
         */
        if h != boothartid {
            sbi_rt::hart_start(h, _start as usize, 0)
                .into_result()
                .unwrap_or_else(|_| panic!("Failed to start hart {}", h));
        }
    }
}

/* main is called by _start, and RISCV calling conventions state that the first two arguments
 * should correspond to registers a0 and a1 if they fit. Conveniently, OpenSBI places the hartid
 * and pointer to DTB in these registers which _start doesn't clobber and are thus accessible
 * from main.
 */
#[no_mangle]
extern "C" fn main(hartid: usize, dtb: *const u8) -> ! {
    match STARTED.compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst) {
        /* If we are the boot hart, parse device tree and start other harts.
         *
         * Cannot rely on checking specific hartid since boot hart
         * is determined via lottery and is non-deterministic.
         */
        Ok(_) => {
            debug_print!("\n\n\n");
            debug_print!("Hack the planet!\n");
            debug_print!("Boot hart: {}\n\n", hartid);

            handle_dtb(dtb);
            debug_print!("\n");

            start_harts(hartid);
        }

        // Otherwise do per-hart setup if needed (if not done in _start)
        Err(_) => {
            debug_print!("Hart {} starting...\n", hartid);
        }
    }

    // Finally do some real work (which all harts are now running in parallel)
    loop {
        riscv::asm::wfi();
    }
}

/* A simple panic handler that will get called any time a panic occurs
 * (either explicitly by us or as generated by the Rust compiler).
 */
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    let message = info.message();
    let file = info.location().map(|loc| loc.file()).unwrap_or("{unknown}");
    let line = info.location().map(|loc| loc.line()).unwrap_or(0);

    debug_print!("{} in {} at line {}\n", message, file, line);
    loop {}
}
