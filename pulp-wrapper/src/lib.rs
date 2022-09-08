#![no_std]
#![feature(allocator_api)]
#![feature(new_uninit)]
extern crate alloc;

use alloc::boxed::Box;
use cipher::{KeyIvInit, StreamCipher, StreamCipherSeek, Unsigned};
use core::{pin::Pin, ptr::NonNull};
use pulp_sdk_rust::*;

use generic_array::GenericArray;

mod buf;
mod cluster;
use buf::{BufAlloc, DmaBuf, SourcePtr};
pub use cluster::Cluster;

const fn parse_cores_u8(s: &str) -> usize {
    let cores = (s.as_bytes()[0] - b'0') as usize;

    if cores.count_ones() != 1 {
        panic!("Unsupported number of cores. Please use a power of 2");
    }
    cores
}

const CORES: usize = parse_cores_u8(core::env!("CORES"));

/// Convenience struct for stream encryption / decryption using the PULP cluster.
/// Supports encryption / decryption directly from ram or L2 memory and manages
/// dma in/out autonomously.
pub struct PulpWrapper<const BUF_LEN: usize> {
    cluster: Cluster,
    cluster_buffer: BufAlloc<BUF_LEN>,
    core_data: NonNull<CoreData<BUF_LEN>>,
}

impl<const BUF_LEN: usize> PulpWrapper<BUF_LEN> {
    /// Initialize the wrapper and allocates necessary buffers in the cluster.
    /// This enables to reuse allocations across calls to [run].
    pub fn new(mut cluster: Cluster) -> Self {
        // Safety: C api will not move out of the returned ptr
        let device_ptr = unsafe { Pin::get_unchecked_mut(cluster.device_mut()) as *mut PiDevice };
        let l1_alloc = ClusterAllocator::new(device_ptr);
        Self {
            cluster_buffer: <BufAlloc<BUF_LEN>>::new(&mut cluster),
            core_data: NonNull::new(Box::leak(Box::new_in(CoreData::empty(), l1_alloc))).unwrap(),
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

        *self.core_data.as_mut() = data;

        pi_cl_team_fork(
            CORES,
            Self::entry_point::<C>,
            self.core_data.as_ptr() as *mut cty::c_void,
        );
    }

    extern "C" fn entry_point<C: StreamCipher + StreamCipherSeek + KeyIvInit>(
        data: *mut cty::c_void,
    ) {
        unsafe {
            let data: &CoreData<BUF_LEN> = &*(data as *const CoreData<BUF_LEN>);
            let CoreData {
                key,
                iv,
                source,
                len,
                l1_alloc,
                loc,
            } = *data;
            let key = GenericArray::from_slice(core::slice::from_raw_parts(key, C::KeySize::USIZE));
            let iv = GenericArray::from_slice(core::slice::from_raw_parts(iv, C::IvSize::USIZE));

            // any lifetime will do as BufAlloc is owned by PulpWrapper
            let l1_alloc = &*l1_alloc;
            let source = SourcePtr::from_raw_parts(source, len);

            let mut cipher = C::new(key, iv);
            let core_id = pi_core_id();

            // To fit all data in L1 cache, we split input in rounds.
            let mut buf = match loc {
                SourceLocation::L2 => {
                    <DmaBuf<CORES, BUF_LEN>>::new_from_l2(source, l1_alloc)
                }
                SourceLocation::Ram(device) => {
                    <DmaBuf<CORES, BUF_LEN>>::new_from_ram(source, l1_alloc, device)
                }
                _ => panic!("unsupported"),
            };
            // If the cipher is producing the keystream in incremental blocks,
            // it's extremely important for efficiency that round_buf_len / cores is a multiple of the block size
            let round_buf_len = <DmaBuf<CORES, BUF_LEN>>::FULL_WORK_BUF_LEN;
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
                cipher.seek(base + past);
                cipher.apply_keystream_inout(buf.get_work_buf());
                buf.advance();
            }

            buf.flush();
        }
    }
}

impl<const BUF_LEN: usize> Drop for PulpWrapper<BUF_LEN> {
    fn drop(&mut self) {
        unsafe {
            let device_ptr = Pin::get_unchecked_mut(self.cluster.device_mut()) as *mut PiDevice;
            let l1_alloc = ClusterAllocator::new(device_ptr);
            let _ = Box::from_raw_in(self.core_data.as_mut(), l1_alloc);
        }
    }
}

struct CoreData<const BUF_LEN: usize> {
    source: *mut u8,
    len: usize,
    l1_alloc: *const BufAlloc<BUF_LEN>,
    key: *const u8,
    iv: *const u8,
    loc: SourceLocation,
}

impl<const BUF_LEN: usize> CoreData<BUF_LEN> {
    fn new(
        source: *mut u8,
        len: usize,
        l1_alloc: *const BufAlloc<BUF_LEN>,
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
            loc,
        }
    }

    fn empty() -> Self {
        Self {
            source: core::ptr::null_mut(),
            len: 0,
            l1_alloc: core::ptr::null(),
            key: core::ptr::null(),
            iv: core::ptr::null(),
            loc: SourceLocation::L1,
        }
    }
}

#[derive(Clone, Copy)]
pub enum SourceLocation {
    L1,
    L2,
    Ram(NonNull<PiDevice>),
}
