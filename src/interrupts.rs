use crate::hlt_loop;
use crate::{gdt, print, println};
use lazy_static::lazy_static;
use pic8259::ChainedPics;
use spin::Mutex;
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame, PageFaultErrorCode};

pub const PIC_1_OFFSET: u8 = 32;
pub const PIC_2_OFFSET: u8 = PIC_1_OFFSET + 8;

pub static PICS: Mutex<ChainedPics> =
    Mutex::new(unsafe { ChainedPics::new(PIC_1_OFFSET, PIC_2_OFFSET) });

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum InterruptIndex {
    Timer = PIC_1_OFFSET,
    Keyboard,
}

impl InterruptIndex {
    fn as_u8(self) -> u8 {
        self as u8
    }

    fn as_usize(self) -> usize {
        self as usize
    }
}

lazy_static! {
    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        idt.breakpoint.set_handler_fn(breakpoint_handler);
        unsafe {
            idt.double_fault
                .set_handler_fn(double_fault_handler)
                .set_stack_index(gdt::DOUBLE_FAULT_IST_INDEX)
        };
        // timer interrupt handler set
        idt[InterruptIndex::Timer.as_usize()].set_handler_fn(timer_interrupt_handler);
        // keyboard interrupt handler set
        idt[InterruptIndex::Keyboard.as_usize()].set_handler_fn(keyboard_interrupt_handler);
        // page fault error handler set
        idt.page_fault.set_handler_fn(page_fault_handler);
        idt
    };
}

unsafe fn notify_end_of_interrupt(interrupt_index: InterruptIndex) {
    PICS.lock().notify_end_of_interrupt(interrupt_index.as_u8());
}

pub fn init_idt() {
    IDT.load(); // load the interrupt descriptor table
}

// =============================================================================================================
// BREAKPOINTS (int3)
extern "x86-interrupt" fn breakpoint_handler(stack_frame: InterruptStackFrame) {
    println!("EXCEPTION BREAKPOINT:\n{:#?}", stack_frame);
}

// -------------------------------------------------------------------------------------------------------------

// =============================================================================================================
// DOUBLE FAULTS
extern "x86-interrupt" fn double_fault_handler(
    stack_frame: InterruptStackFrame,
    _error_code: u64,
) -> ! {
    panic!("EXCEPTION: DOUBLE FAULT:\n:{:#?}", stack_frame);
}

// -------------------------------------------------------------------------------------------------------------

// =============================================================================================================
// TIMER INTERRUPTS
extern "x86-interrupt" fn timer_interrupt_handler(_stack_frame: InterruptStackFrame) {
    print!(".");

    unsafe {
        notify_end_of_interrupt(InterruptIndex::Timer);
    }
}

// -------------------------------------------------------------------------------------------------------------
// =============================================================================================================
// KEYBOARD INTERRUPTS
extern "x86-interrupt" fn keyboard_interrupt_handler(_stack_frame: InterruptStackFrame) {
    use pc_keyboard::{layouts, DecodedKey, HandleControl, Keyboard, ScancodeSet1};
    use x86_64::instructions::port::Port;

    lazy_static! {
        static ref KEYBOARD: Mutex<Keyboard<layouts::Us104Key, ScancodeSet1>> =
            Mutex::new(Keyboard::new(HandleControl::Ignore));
    }

    let mut keyboard = KEYBOARD.lock();
    let mut keyboard_port = Port::new(0x60);
    let scancode: u8 = unsafe { keyboard_port.read() };

    if let Ok(Some(key_event)) = keyboard.add_byte(scancode) {
        if let Some(key) = keyboard.process_keyevent(key_event) {
            match key {
                DecodedKey::RawKey(raw_key) => print!("{:?}", raw_key),
                DecodedKey::Unicode(char) => print!("{}", char),
            }
        }
    }

    unsafe {
        notify_end_of_interrupt(InterruptIndex::Keyboard);
    }
}

extern "x86-interrupt" fn page_fault_handler(
    stack_frame: InterruptStackFrame,
    error_code: PageFaultErrorCode,
) {
    use x86_64::registers::control::Cr2;

    println!("EXCEPTION: PAGE FAULT");
    println!("Accessed Address: {:?}", Cr2::read());
    println!("Error Code: {:?}", error_code);
    println!("{:#?}", stack_frame);
    hlt_loop();
}

//------------------------------------------------------------------------------------------

// =======================================================================================//
// ------------------------------->> END OF FILE <<---------------------------------------//
// =======================================================================================//
