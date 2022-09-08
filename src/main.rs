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
    toy_os::hlt_loop();
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    toy_os::test_panic_handler(info)
}

#[no_mangle]
pub extern "C" fn _start() -> ! {
    toy_os::init();

    #[cfg(test)]
    test_main();

    #[cfg(not(test))]
    use x86_64::registers::control::Cr3;
    let (level_4_page_table, _) = Cr3::read();
    println!("Level 4 page table at: {:?}", level_4_page_table.start_address());

    toy_os::hlt_loop();
}