#![no_std]
#![no_main]

#![feature(custom_test_frameworks)]
#![test_runner(crate::test::test_runner)]
#![reexport_test_harness_main = "test_main"]

mod vga_buffer;
mod serial;
mod test;

use core::panic::PanicInfo;
use crate::test::{exit_qemu, QemuExitCode};

#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    loop {}
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    serial_println!("[failed]\n");
    serial_println!("Error: {}\n", info);
    exit_qemu(QemuExitCode::Failed);
    loop {}
}

#[no_mangle]
pub extern "C" fn _start() -> ! {
    println!("hello world");

    #[cfg(test)]
    test_main();

    loop {}
}