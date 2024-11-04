use log::error;
use log::debug;
use log::{file, line};
use crate::config::KHEAP_SIZE;
use crate::sync::mutex::NoInterruptSpinLock;
use buddy_system_allocator::Heap;
use core::{
    alloc::{GlobalAlloc, Layout},
    ptr::NonNull,
};

// 函数申明-----------------------------------------------------------------
pub fn init_heap();
pub fn handle_alloc_error(layout: Layout) -> !;

// 初始化全局堆分配器--------------------------------------------------------
#[global_allocator]
static KHEAP_ALLOCATOR : KHeapAllocator;

// 内核堆大小和开辟内核堆, 静态数组大小为kernel size---------------------------
static mut KHEAP_SPACE : [u8; KHEAP_SIZE] = [0; KHEAP_SIZE];

// 内核堆分配器--------------------------------------------------------------
struct KHeapAllocator(NoInterruptSpinLock<Heap<32>>);

impl KHeapAllocator {
    pub fn new() -> Self {
        Self(NoInterruptSpinLock::new(Heap::empty()))
    }
}

unsafe impl GlobalAlloc for KHeapAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        match self.0.lock().alloc(layout) {
            Some(allocation) => allocation.as_ptr(),
            None => {
                error!("Kernel heap allocation failed: {:?}", layout);
                core::ptr::null_mut()
            }
        }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        self.0.lock().dealloc(NonNull::new_unchecked(ptr), layout)
    }
}

// 初始化堆，在main函数中内核启动过程中调用--------------------------------------
pub fn init_heap() {
    unsafe {
        let start_ptr = KHEAP_SPACE.as_ptr();
        let end_ptr   = start_ptr.add(KHEAP_SIZE);
        let start     = start_ptr as usize;
        let end       = end_ptr as usize;
        KHEAP_ALLOCATOR.0.lock().add_to_heap(start, end);
        println!("[kernel] Initialize kernel heap: {:?} - {:?}", start, end);
    }
}

// 处理分配失败的情况-----------------------------------------------------------
pub fn handle_alloc_error(layout: Layout) -> ! {
    let size = layout.get_size();
    let align = layout.get_align();
    let file = file!();
    let line = line!();
    error!(
        "Kernel heap allocation failed. Requested size: {}, Alignment: {}. Details: {:?}. Location: {}:{}",
        size, alignment, layout, file, line
    );
    panic!("Kernel heap allocation failed!");
}