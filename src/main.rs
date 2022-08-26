#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(toy_os::test_runner)]
#![reexport_test_harness_main = "test_main"]

use core::panic::PanicInfo;
#[cfg(not(test))]
use toy_os::println;

/// 正常panic handler
#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    loop {}
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    toy_os::test_panic_handler(info)
}

#[no_mangle]
pub extern "C" fn _start() -> ! {
    toy_os::init();
    x86_64::instructions::interrupts::int3();

    #[cfg(test)]
    test_main();

    loop {}
}