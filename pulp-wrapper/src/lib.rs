#![no_std]
#![feature(allocator_api)]
#![feature(new_uninit)]
extern crate alloc;

use cipher::{KeyIvInit, StreamCipher, StreamCipherSeek, Unsigned};
use core::ptr::NonNull;
use pulp_sdk_rust::*;

use generic_array::GenericArray;

mod buf;
use buf::{BufAlloc, DmaBuf, SourcePtr};

/// Convenience struct for stream encryption / decryption using the PULP cluster.
/// Supports encryption / decryption directly from ram or L2 memory and manages
/// dma in/out autonomously.
pub struct PulpWrapper<const CORES: usize, const BUF_LEN: usize> {
    cluster: Cluster<CORES>,
    // The correct lifetime here would be 'self if we could write it
    // As long as this is never exposed outside and we know our use does not
    // result in invalid references it's fine to use 'static
    cluster_buffer: BufAlloc<'static, BUF_LEN>,
}

impl<const CORES: usize, const BUF_LEN: usize> PulpWrapper<CORES, BUF_LEN> {
    /// Initialize the wrapper and allocates necessary buffers in the cluster.
    /// This is to reuse allocations across calls to [run].
    pub fn new(cluster: Cluster<CORES>) -> Self {
        let buffer = <BufAlloc<BUF_LEN>>::new(&cluster);
        Self {
            cluster_buffer: unsafe {
                core::mem::transmute::<BufAlloc<'_, BUF_LEN>, BufAlloc<'static, BUF_LEN>>(buffer)
            },
            cluster,
        }
    }

    /// Encrypt / decrypt data in [source] with given key and iv
    ///
    /// # Safety:
    /// * source location must be correctly specified in [loc]
    /// * if present, ram device pointer must be valid to read for the whole duration
    pub unsafe fn run<C: StreamCipher + StreamCipherSeek + KeyIvInit>(
        &mut self,
        source: &mut [u8],
        key: &GenericArray<u8, C::KeySize>,
        iv: &GenericArray<u8, C::IvSize>,
        loc: SourceLocation,
    ) {
        let data = CoreData::new(
            source.as_mut_ptr(),
            source.len(),
            &self.cluster_buffer,
            key.as_ptr(),
            iv.as_ptr(),
            loc,
        );
        use alloc::boxed::Box;
        let data = Box::leak(Box::new_in(data , self.cluster.l1_allocator())) as *mut _ as *mut cty::c_void;
        pi_cl_team_fork(CORES, Self::entry_point::<C>, data);
    }

    extern "C" fn entry_point<C: StreamCipher + StreamCipherSeek + KeyIvInit>(
        data: *mut cty::c_void,
    ) {
        unsafe {
            let CoreData {
                key,
                iv,
                source,
                len,
                l1_alloc,
                loc,
            } = *(data as *mut CoreData<BUF_LEN>);
            let key = GenericArray::from_slice(core::slice::from_raw_parts(key, C::KeySize::USIZE));
            let iv = GenericArray::from_slice(core::slice::from_raw_parts(iv, C::IvSize::USIZE));

            // any lifetime will do as BufAlloc is owned by PulpWrapper
            let l1_alloc = &*l1_alloc;
            let source = SourcePtr::from_raw_parts(source, len);

            let mut cipher = C::new(key, iv);
            let core_id = pi_core_id();

            // To fit all data in L1 cache, we split input in rounds.
            let mut buf = match loc {
                SourceLocation::L2 => <DmaBuf<CORES, BUF_LEN>>::new_from_l2(source, l1_alloc),
                SourceLocation::Ram(device) => {
                    <DmaBuf<CORES, BUF_LEN>>::new_from_ram(source, l1_alloc, device)
                }
                _ => panic!("unsupported"),
            };
            // If the cipher is producing the keystream in incremental blocks,
            // it's extremely important for efficiency that round_buf_len / cores is a multiple of the block size
            let round_buf_len = <DmaBuf<CORES, BUF_LEN>>::FULL_WORK_BUF_LEN;
            debug_assert_eq!(round_buf_len % CORES, 0);
            let full_rounds = len / round_buf_len;
            let base = core_id * (round_buf_len / CORES);
            let mut past = 0;

            for _ in 0..full_rounds {
                cipher.seek(base + past);
                cipher.apply_keystream_inout(buf.get_work_buf());
                past += round_buf_len;
                buf.advance();
            }

            // handle remaining buffer
            if len > past {
                let base = (((len - past) + CORES - 1) / CORES) * core_id;
                cipher.seek(base + past);
                cipher.apply_keystream_inout(buf.get_work_buf());
                buf.advance();
            }

            buf.flush();
        }
    }
}

struct CoreData<const BUF_LEN: usize> {
    source: *mut u8,
    len: usize,
    l1_alloc: *const BufAlloc<'static, BUF_LEN>,
    key: *const u8,
    iv: *const u8,
    loc: SourceLocation,
}

// This is not safe in general but we promise we won't abuse it
unsafe impl<const BUF_LEN: usize> Send for CoreData<BUF_LEN> {}
unsafe impl<const BUF_LEN: usize> Sync for CoreData<BUF_LEN> {}

impl<const BUF_LEN: usize> CoreData<BUF_LEN> {
    fn new(
        source: *mut u8,
        len: usize,
        l1_alloc: *const BufAlloc<'static, BUF_LEN>,
        key: *const u8,
        iv: *const u8,
        loc: SourceLocation,
    ) -> Self {
        Self {
            source,
            len,
            l1_alloc,
            key,
            iv,
            loc
        }
    }
}

#[derive(Clone, Copy)]
pub enum SourceLocation {
    L1,
    L2,
    Ram(NonNull<PiDevice>),
}
