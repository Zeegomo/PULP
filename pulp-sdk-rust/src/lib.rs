#![no_std]
#![feature(allocator_api)]
#![feature(alloc_error_handler)]
#![feature(nonnull_slice_from_raw_parts)]
extern crate alloc as core_alloc;

// Should use a more specific target triple like riscv32imcXpulp-unknown-pulp-{abi}
// but we haven't added support for that in the Rust compiler yet.
#[cfg(not(target_arch = "riscv32"))]
compile_error!("unsupported target");

mod alloc;
mod bindings;

pub use alloc::*;
pub use bindings::*;
