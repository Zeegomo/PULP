#![no_std]
#![feature(allocator_api)]
#![feature(default_alloc_error_handler)]
#![feature(new_uninit)]
extern crate alloc;

use alloc::boxed::Box;
use cipher::{IvSizeUser, KeySizeUser, Unsigned};
use core::ptr::NonNull;
use generic_array::GenericArray;
use pulp_sdk_rust::{abort_all, GlobalAllocator, PiDevice, Cluster};
use pulp_wrapper::{PulpWrapper, SourceLocation};
// This should not actually be used, as it's not clear from the context what the default allocation is
#[global_allocator]
static DEFAULT_ALLOCATOR: GlobalAllocator = GlobalAllocator;

use core::panic::PanicInfo;

#[panic_handler]
fn panic_handler(_info: &PanicInfo) -> ! {
    unsafe { abort_all() };
    loop {}
}

#[repr(C)]
pub enum Cipher {
    ChaCha20Pulp,
    ChaCha20,
    Aes128Ctr,
}

macro_rules! extract_key_iv {
    ($cipher:ty, $key:expr, $iv:expr) => {{
        let key = GenericArray::from_slice(core::slice::from_raw_parts(
            $key,
            <$cipher as KeySizeUser>::KeySize::USIZE,
        ));
        let iv = GenericArray::from_slice(core::slice::from_raw_parts(
            $iv,
            <$cipher as IvSizeUser>::IvSize::USIZE,
        ));
        (key, iv)
    }};
}

type Aes128Ctr = ctr::Ctr32LE<aes::Aes128>;
const CLUSTER_L1_BUFFER_LEN: usize = 8192;
const CORES: usize = parse_cores_u8(core::env!("CORES"));

const fn parse_cores_u8(s: &str) -> usize {
    let cores = (s.as_bytes()[0] - b'0') as usize;

    if cores.count_ones() != 1 {
        panic!("Unsupported number of cores. Please use a power of 2");
    }
    cores
}




/// Initialize the cluster and the cluster wrapper wrapper in L2 memory
#[no_mangle]
pub extern "C" fn cluster_init() -> *mut cty::c_void {
    let cluster = <Cluster<CORES>>::new().unwrap();
    let wrapper = Box::new_in(
        <PulpWrapper<CORES,CLUSTER_L1_BUFFER_LEN>>::new(cluster),
        pulp_sdk_rust::L2Allocator,
    );
    Box::into_raw(wrapper) as *mut cty::c_void
}

/// Encrypt / decrypt using the provided cipher
///
/// # Safety:
/// * data must be valid to read / write for len bytes and must be in L2 memory
/// * key must be valid to read for: 32 bytes
/// * iv must be valid to read for 12 bytes
/// * wrapper must be a valid pointer to an initialized PULP Wrapper allocated by this library
#[no_mangle]
pub unsafe extern "C" fn encrypt(
    data: *mut u8,
    len: usize,
    key: *const u8,
    iv: *const u8,
    wrapper: *mut cty::c_void,
    ram_device: *mut PiDevice,
    cipher: Cipher,
) {
    let wrapper = (wrapper as *mut PulpWrapper<CORES, CLUSTER_L1_BUFFER_LEN>)
        .as_mut()
        .unwrap();
    let data = core::slice::from_raw_parts_mut(data, len);
    let location = if let Some(device) = NonNull::new(ram_device) {
        SourceLocation::Ram(device)
    } else {
        SourceLocation::L2
    };
    match cipher {
        Cipher::ChaCha20 => {
            let (key, iv) = extract_key_iv!(chacha20_orig::ChaCha20, key, iv);
            wrapper.run::<chacha20_orig::ChaCha20>(data, key, iv, location)
        }
        Cipher::ChaCha20Pulp => {
            let (key, iv) = extract_key_iv!(chacha20::ChaCha20, key, iv);
            wrapper.run::<chacha20::ChaCha20>(data, key, iv, location)
        }
        Cipher::Aes128Ctr => {
            let (key, iv) = extract_key_iv!(Aes128Ctr, key, iv);
            wrapper.run::<Aes128Ctr>(data, key, iv, location)
        }
    }
}

/// Clean up resources used by the PULP wrapper
///
/// Safety: wrapper must be a valid pointer to an initialized PULP wrapper
#[no_mangle]
pub unsafe extern "C" fn cluster_close(wrapper: *mut cty::c_void) {
    let _wrapper = Box::from_raw_in(wrapper as *mut PulpWrapper<CORES,CLUSTER_L1_BUFFER_LEN>, pulp_sdk_rust::L2Allocator);
}

/// Encrypt data serially using the unmodified version of this library
///
/// # Safety:
/// * data must be valid to read / write for len bytes and must be in L2 memory
/// * key must be valid to read for 32 bytes
/// * iv must be valid to read for 12 bytes
#[no_mangle]
pub extern "C" fn encrypt_serial_orig(data: *mut u8, len: usize, key: *const u8, iv: *const u8) {
    use chacha20_orig::cipher::StreamCipher;
    use chacha20_orig::*;
    use cipher::KeyIvInit;
    let data = unsafe { core::slice::from_raw_parts_mut(data, len) };
    let key = Key::from_slice(unsafe { core::slice::from_raw_parts(key, 32) });
    let iv = Nonce::from_slice(unsafe { core::slice::from_raw_parts(iv, 12) });
    let mut chacha = chacha20_orig::ChaCha20::new(key, iv);
    chacha.apply_keystream(data);
}
