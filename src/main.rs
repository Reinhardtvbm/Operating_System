#![no_std]
#![no_main]

use core::panic::PanicInfo;

mod vga_buffer;

/// self defined panic handler function
#[panic_handler] // -> ! means never returns
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

static HELLO: &[u8] = b"Hello World!";

#[no_mangle]
pub extern "C" fn _start() -> ! {
    use core::fmt::Write;

    vga_buffer::VGA_WRITER
        .lock()
        .write_str("Global static buffer here!")
        .unwrap();

    loop {}
}
