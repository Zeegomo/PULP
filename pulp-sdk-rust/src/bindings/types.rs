#[repr(C)]
pub struct PiClDmaCmd {
    id: cty::c_int,
    next: *mut Self,
}

impl PiClDmaCmd {
    pub fn new() -> Self {
        Self {
            id: 0,
            next: core::ptr::null_mut(),
        }
    }
}

impl Default for PiClDmaCmd {
    fn default() -> Self {
        Self::new()
    }
}

#[allow(non_camel_case_types)]
#[repr(C)]
pub enum PiClDmaDirE {
    PI_CL_DMA_DIR_LOC2EXT = 0,
    PI_CL_DMA_DIR_EXT2LOC = 1,
}

#[repr(C)]
pub struct PiDevice {
    api: *mut PiDeviceApi,
    config: *mut cty::c_void,
    data: *mut cty::c_void,
}

impl PiDevice {
    pub fn uninit() -> Self {
        Self {
            api: core::ptr::null_mut() as *mut PiDeviceApi,
            config: core::ptr::null_mut() as *mut cty::c_void,
            data: core::ptr::null_mut() as *mut cty::c_void,
        }
    }
}

#[repr(C)]
pub struct PiClRamReq {
    device: *mut PiDevice,
    addr: *mut cty::c_void,
    ram_addr: u32,
    size: u32,
    stride: u32,
    length: u32,
    event: PiTask,
    next: *mut Self,
    done: u8,
    cid: cty::c_char,
    ext2loc: cty::c_char,
    is_2d: cty::c_char,
}

impl PiClRamReq {
    pub fn new(device: *mut PiDevice) -> Self {
        Self {
            device,
            addr: core::ptr::null_mut(),
            ram_addr: 0,
            size: 0,
            stride: 0,
            length: 0,
            event: PiTask::new(),
            next: core::ptr::null_mut(),
            done: 0,
            cid: 0,
            ext2loc: 0,
            is_2d: 0,
        }
    }

    pub fn is_in_transfer(&self) -> bool {
        self.ext2loc != 0
    }

    pub fn device(&self) -> *mut PiDevice {
        self.device
    }
}

const PI_TASK_IMPLEM_NB_DATA: usize = 8;

#[derive(Default)]
#[repr(C)]
#[repr(packed)]
struct PiTaskImplem {
    time: cty::c_uint,
}

#[repr(C)]
pub struct PiTask {
    // Warning, might be accessed inline in asm, and thus can not be moved
    next: *mut Self,
    arg: [usize; 4],
    done: i8,
    id: cty::c_int,
    data: [u32; PI_TASK_IMPLEM_NB_DATA],
    implem: PiTaskImplem,
}

impl PiTask {
    fn new() -> Self {
        Self {
            next: core::ptr::null_mut(),
            arg: [0; 4],
            done: 0,
            id: 0,
            data: [0; PI_TASK_IMPLEM_NB_DATA],
            implem: PiTaskImplem::default(),
        }
    }
}

#[repr(C)]
pub struct PiClusterConf {
    // do not move this one, might be accessed in various hackish way
    device_type: PiDeviceType,
    /// Cluster ID, starting from 0
    id: cty::c_int,
    /// Reserved for internal usage
    heap_start: *mut cty::c_void,
    /// Reserved for internal usage
    heap_size: u32,
    /// Reserved for internal usage
    event_kernel: *mut PmsisEventKernelWrap,
    /// Additional flags
    flags: PiClusterFlags,
}

impl PiClusterConf {
    pub fn uninit() -> Self {
        Self {
            device_type: PiDeviceType::PiDeviceUnkwnType,
            id: 0,
            heap_start: core::ptr::null_mut() as *mut cty::c_void,
            heap_size: 0,
            event_kernel: core::ptr::null_mut() as *mut PmsisEventKernelWrap,
            flags: PiClusterFlags::PiClusterFlagsForkBased,
        }
    }
}

#[repr(C)]
pub enum PiClusterFlags {
    PiClusterFlagsForkBased = 0,
    PiClusterFlagsTaskBased = 1,
}

#[repr(C)]
pub enum PiDeviceType {
    PiDeviceUnkwnType,
    PiDeviceClusterType,
    PiDeviceHyperbusType,
    PiDeviceSpiType,
    PiDeviceCpiType,
    PiDeviceI2cType,
    PiDeviceGpioType,
    PiDevicePwmType,
}

#[repr(C)]
pub struct PiClusterTask {
    // entry function and its argument(s)
    entry: extern "C" fn(arg: *mut cty::c_void),
    arg: *mut cty::c_void,
    // pointer to first stack, and size for each cores
    stacks: *mut cty::c_void,
    stack_size: cty::uint32_t,
    slave_stack_size: cty::uint32_t,
    // Number of cores to be activated
    nb_cores: cty::c_int,
    // callback called at task completion
    completion_callback: *mut PiTaskOpaque,
    stack_allocated: cty::c_int,
    // to implement a fifo
    next: *mut Self,

    core_mask: cty::c_int,
}

extern "C" fn noop(_: *mut cty::c_void) {}

impl PiClusterTask {
    pub fn uninit() -> Self {
        Self {
            entry: noop,
            arg: core::ptr::null_mut() as *mut cty::c_void,
            stacks: core::ptr::null_mut() as *mut cty::c_void,
            stack_size: 0,
            slave_stack_size: 0,
            nb_cores: 0,
            completion_callback: core::ptr::null_mut() as *mut PiTaskOpaque,
            stack_allocated: 0,
            next: core::ptr::null_mut() as *mut Self,
            core_mask: 0,
        }
    }
}

// Opaque structsself.core_data.as_mut()
// Not really fully opaque in C but they are not used by Rust code and it's easier to tream them as such

#[repr(C)]
pub struct PmsisEventKernelWrap {
    // Private field to avoid instantiation outside of this module
    _data: [u8; 0],
    // Do not let the compiler assume stuff it shouldn't
    _marker: core::marker::PhantomData<(*mut u8, core::marker::PhantomPinned)>,
}

#[repr(C)]
pub struct PiDeviceApi {
    // Private field to avoid instantiation outside of this module
    _data: [u8; 0],
    // Do not let the compiler assume stuff it shouldn't
    _marker: core::marker::PhantomData<(*mut u8, core::marker::PhantomPinned)>,
}

#[repr(C)]
pub struct PiTaskOpaque {
    // Private field to avoid instantiation outside of this module
    _data: [u8; 0],
    // Do not let the compiler assume stuff it shouldn't
    _marker: core::marker::PhantomData<(*mut u8, core::marker::PhantomPinned)>,
}
