#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(os::test_runner)]
#![reexport_test_harness_main = "test_main"]

use bootloader::{entry_point, BootInfo};
use core::panic::PanicInfo;
use os::{memory, println};
use x86_64::structures::paging::Translate;

entry_point!(kernel_main);

fn kernel_main(boot_info: &'static BootInfo) -> ! {
    use x86_64::VirtAddr;

    println!("Hello OS!");

    // INITIALISATION
    os::init();

    let mapper = unsafe { memory::init(VirtAddr::new(boot_info.physical_memory_offset)) };

    let addresses = [
        // the identity-mapped vga buffer page
        0xb8000,
        // some code page
        0x201008,
        // some stack page
        0x0100_0020_1a10,
        // virtual address mapped to physical address 0
        boot_info.physical_memory_offset,
    ];

    addresses.into_iter().for_each(|address| {
        let virtual_addr = VirtAddr::new(address);
        let physical_addr = mapper.translate_addr(virtual_addr);
        println!("{:?} -> {:?}", virtual_addr, physical_addr)
    });

    // TESTING
    #[cfg(test)]
    test_main();

    // END :)
    println!("It did not crash!");

    os::hlt_loop();
}

/// self defined panic handler function
#[cfg(not(test))]
#[panic_handler] // -> ! means never returns
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    os::hlt_loop();
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    os::test_panic_handler(info);
}
