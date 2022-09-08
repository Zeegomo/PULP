#![no_std]
#![feature(allocator_api)]
#![feature(alloc_error_handler)]
#![feature(nonnull_slice_from_raw_parts)]

use core::sync::atomic::*;
extern crate alloc as core_alloc;
use core_alloc::string::String;

// Should use a more specific target triple like riscv32imcXpulp-unknown-pulp-{abi}
// but we haven't added support for that in the Rust compiler yet.
#[cfg(not(target_arch="riscv32"))]
compile_error!("unsupported target");


mod alloc;
mod bindings;

pub use alloc::*;
pub use bindings::*;

pub fn pi_cl_dma_cmd(
    ext: *mut u8,
    loc: *mut u8,
    size: usize,
    dir: PiClDmaDirE,
    cmd: &mut PiClDmaCmd,
) {
    // Prevent the compiler from reordering a volatile read / write
    // not sure this is really needed given the call to an opaque function
    fence(Ordering::Release);
    unsafe {
        pi_cl_dma_cmd_wrap(
            ext as usize as u32,
            loc as usize as u32,
            size as u32,
            dir,
            cmd as *mut PiClDmaCmd,
        )
    }
}

pub fn pi_cl_dma_wait(copy: &mut PiClDmaCmd) {
    // Prevent the compiler from reordering a volatile read / write
    // not sure this is really needed given the call to an opaque function
    fence(Ordering::Acquire);
    unsafe { pi_cl_dma_wait_wrap(copy as *mut PiClDmaCmd as *mut cty::c_void) }
}

pub fn pi_cl_ram_read_wait(req: &mut PiClRamReq) {
    unsafe { pi_cl_ram_read_wait_wrap(req as *mut PiClRamReq) }
}

pub fn pi_cl_ram_write_wait(req: &mut PiClRamReq) {
    unsafe { pi_cl_ram_write_wait_wrap(req as *mut PiClRamReq) }
}

pub fn pi_cl_ram_read(
    device: *mut PiDevice,
    pi_ram_addr: *mut u8,
    addr: *mut u8,
    size: usize,
    req: &mut PiClRamReq,
) {
    unsafe {
        pi_cl_ram_read_wrap(
            device,
            pi_ram_addr as cty::uint32_t,
            addr as *mut cty::c_void,
            size as cty::uint32_t,
            req as *mut PiClRamReq,
        )
    }
}

pub fn pi_cl_ram_write(
    device: *mut PiDevice,
    pi_ram_addr: *mut u8,
    addr: *mut u8,
    size: usize,
    req: &mut PiClRamReq,
) {
    unsafe {
        pi_cl_ram_write_wrap(
            device,
            pi_ram_addr as cty::uint32_t,
            addr as *mut cty::c_void,
            size as cty::uint32_t,
            req as *mut PiClRamReq,
        )
    }
}

pub fn print(mut msg: String) {
    unsafe { 
        let v = msg.as_mut_vec();
        let len = v.len();
        v[len-1] = 0;
        print_wrap(v.as_ptr() as *const cty::c_char) 
    }
}

// TODO: compiler fence?
pub fn pi_cl_team_barrier() {
    unsafe { pi_cl_team_barrier_wrap() }
}

// TODO: compiler fence?
// TODO: rewrite this as a safe function
pub unsafe fn pi_cl_team_fork(
    num_cores: usize,
    cluster_fn: extern "C" fn(*mut cty::c_void),
    args: *mut cty::c_void,
) {
    pi_cl_team_fork_wrap(num_cores, cluster_fn, args);
}
