// Memory!

use x86_64::structures::paging::OffsetPageTable;
use x86_64::{structures::paging::PageTable, PhysAddr, VirtAddr};

pub unsafe fn init(physical_memory_offset: VirtAddr) -> OffsetPageTable<'static> {
    let l4_table = active_level4_page_table(physical_memory_offset);
    OffsetPageTable::new(l4_table, physical_memory_offset)
}

/// Returns a mutable reference to the active level 4 table.
///
/// This function is unsafe because the caller must guarantee that the
/// complete physical memory is mapped to virtual memory at the passed
/// `physical_memory_offset`. Also, this function must be only called once
/// to avoid aliasing `&mut` references (which is undefined behavior).
unsafe fn active_level4_page_table(physical_memory_offset: VirtAddr) -> &'static mut PageTable {
    // note that PhysAddr and VirtAddr are tuple struct wrappers of u64

    use x86_64::registers::control::Cr3; // contains physical address of highest level page table

    //  l4_table_frame is of type PhysFrame { start_address: PhysAddr, size: PhantomData<S> }
    let (l4_table_frame, _) = Cr3::read();

    let frame_phys_address = l4_table_frame.start_address();
    let virt = physical_memory_offset + frame_phys_address.as_u64();

    &mut *virt.as_mut_ptr() // unsafe
}

pub unsafe fn get_physical_addr(
    virtual_address: VirtAddr,
    physical_memory_offset: VirtAddr,
) -> Option<PhysAddr> {
    // inner function contains explicit unsafes
    translate(virtual_address, physical_memory_offset)
}

pub fn translate(virtual_address: VirtAddr, physical_memory_offset: VirtAddr) -> Option<PhysAddr> {
    use x86_64::registers::control::Cr3;
    use x86_64::structures::paging::page_table::FrameError;

    let (active_l4_table_frame, _) = Cr3::read();

    let table_indices = [
        virtual_address.p4_index(),
        virtual_address.p3_index(),
        virtual_address.p2_index(),
        virtual_address.p1_index(),
    ];

    let mut frame = active_l4_table_frame;

    for index in table_indices {
        let start_address = frame.start_address().as_u64();

        // makes more sense to write '(start_address + physical_memory_offset).as_ptr()',
        // but Add<PhysAddr> for u64 not implemented in x86_64, only Add<u64> for PhysAddr
        let table_ref: &PageTable =
            unsafe { &*((physical_memory_offset + start_address).as_ptr()) };

        let entry = &table_ref[index];

        frame = match entry.frame() {
            Ok(phys_frame) => phys_frame,
            Err(FrameError::HugeFrame) => panic!("Huge frames not supported!"),
            Err(FrameError::FrameNotPresent) => return None,
        }
    }

    Some(frame.start_address() + u64::from(virtual_address.page_offset()))
}
