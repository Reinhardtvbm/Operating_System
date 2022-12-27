#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(os::test_runner)]
#![reexport_test_harness_main = "test_main"]

use core::panic::PanicInfo;
use os::print;

#[no_mangle] // don't mangle the name of this function
pub extern "C" fn _start() -> ! {
    test_main();

    loop {}
}

// fn test_runner(tests: &[&dyn Fn()]) {
//     unimplemented!();
// }

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    os::test_panic_handler(info);
}

#[test_case]
fn vga_write_robust() {
    for number in 0..2000 {
        print!("{} ", number);
    }
}
