#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(crate::test_runner)]
#![reexport_test_harness_main = "test_main"]

use core::panic::PanicInfo;

mod vga_buffer;

#[cfg(test)]
fn test_runner(tests: &[&dyn Fn()]) {
    println!("Running {} tests", tests.len());
    for test in tests {
        test();
    }
}

/// self defined panic handler function
#[panic_handler] // -> ! means never returns
fn panic(info: &PanicInfo) -> ! {
    print!("\n{}", info);

    loop {}
}

static HELLO: &[u8] = b"Hello World!";

#[no_mangle]
pub extern "C" fn _start() -> ! {
    println!("Hello OS!");

    #[cfg(test)]
    test_main();

    loop {}
}

#[test_case]
fn test_test_case() {
    print!("testing the testing... ");
    assert_eq!(1, 1);
    println!("[ok]");
}
