use {
    conquer_once::spin::Lazy,
    frame_allocator::FrameAllocator,
    spinning_top::{Spinlock, SpinlockGuard},
    uefi::service::boot::MemoryDescriptor,
    x86_64::structures::paging::Size4KiB,
};

const REASONABLE_NUM_DESCRIPTORS: usize = 256;

static FRAME_ALLOCATOR: Lazy<Spinlock<FrameAllocator<Size4KiB, REASONABLE_NUM_DESCRIPTORS>>> =
    Lazy::new(|| Spinlock::new(FrameAllocator::new()));

pub fn frame_allocator<'a>(
) -> SpinlockGuard<'a, FrameAllocator<Size4KiB, REASONABLE_NUM_DESCRIPTORS>> {
    let f = FRAME_ALLOCATOR.try_lock();

    f.expect("Failed to acquire the lock of the frame allocator.")
}

pub(super) fn init(mmap: &[MemoryDescriptor]) {
    frame_allocator().init(mmap);

    #[cfg(test_on_qemu)]
    tests::main();
}

#[cfg(test_on_qemu)]
mod tests {
    use {
        super::frame_allocator,
        crate::NumOfPages,
        x86_64::structures::paging::frame::{PhysFrame, PhysFrameRange},
    };

    pub(super) fn main() {
        allocate_single_page_and_dealloc();
    }

    fn allocate_single_page_and_dealloc() {
        let p = alloc(NumOfPages::new(1));
        let p = p.expect("Failed to allocate a page.");

        dealloc(p.start);
    }

    #[must_use]
    fn alloc(n: NumOfPages) -> Option<PhysFrameRange> {
        frame_allocator().alloc(n)
    }

    fn dealloc(f: PhysFrame) {
        frame_allocator().dealloc(f);
    }
}
