use crate::{pmsis_l1_free, pmsis_l1_malloc, pmsis_l2_free, pmsis_l2_malloc};
use core::alloc::{AllocError, Allocator, GlobalAlloc, Layout};
use core::ptr::NonNull;

const L2_ALIGN: usize = 4;
const CLUSTER_L1_ALIGN: usize = 4;

/// Allocate memory on chip L2 memory
/// Wrapper around `pi_l2_malloc` and `pi_l2_free`
pub struct L2Allocator;

unsafe impl Allocator for L2Allocator {
    fn allocate(&self, layout: Layout) -> Result<NonNull<[u8]>, AllocError> {
        if layout.align() > L2_ALIGN {
            // TODO: use pi_l2_malloc_align()
            return Err(AllocError);
        }
        let ptr = pmsis_l2_malloc(layout.size().try_into().map_err(|_| AllocError)?) as *mut u8;
        NonNull::new(ptr)
            .map(|ptr| NonNull::slice_from_raw_parts(ptr, layout.size()))
            .ok_or(AllocError)
    }
    unsafe fn deallocate(&self, ptr: NonNull<u8>, layout: Layout) {
        pmsis_l2_free(ptr.as_ptr() as *mut cty::c_void, layout.size() as u32);
    }
}

pub struct GlobalAllocator;

unsafe impl GlobalAlloc for GlobalAllocator {
    unsafe fn alloc(&self, _layout: Layout) -> *mut u8 {
        core::ptr::null_mut()
    }
    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {}
}

#[derive(Clone, Copy)]
pub struct ClusterAllocator<'a> {
    _marker: core::marker::PhantomData<&'a u8>,
}

impl<'a> ClusterAllocator<'a> {
    pub fn new() -> Self {
        Self {
            _marker: core::marker::PhantomData,
        }
    }
}

unsafe impl<'a> Allocator for ClusterAllocator<'a> {
    fn allocate(&self, layout: Layout) -> Result<NonNull<[u8]>, AllocError> {
        if layout.align() > CLUSTER_L1_ALIGN {
            // TODO: use pi_l2_malloc_align()
            return Err(AllocError);
        }
        let ptr = unsafe { pmsis_l1_malloc(layout.size().try_into().map_err(|_| AllocError)?) }
            as *mut u8;
        NonNull::new(ptr)
            .map(|ptr| NonNull::slice_from_raw_parts(ptr, layout.size()))
            .ok_or(AllocError)
    }
    unsafe fn deallocate(&self, ptr: NonNull<u8>, layout: Layout) {
        pmsis_l1_free(ptr.as_ptr() as *mut cty::c_void, layout.size() as u32);
    }
}

#[alloc_error_handler]
fn abort_on_alloc_err(_: core::alloc::Layout) -> ! {
    unsafe {
        crate::abort_all();
    }
    loop {}
}
