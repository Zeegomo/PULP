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



extern "C" {
    pub fn pi_cl_dma_cmd_wrap(
        ext: cty::uint32_t,
        loc: cty::uint32_t,
        size: cty::uint32_t,
        dir: PiClDmaDirE,
        cmd: *mut PiClDmaCmd,
    );

    pub fn pi_cl_dma_wait_wrap(copy: *mut cty::c_void);

    pub fn pi_cl_ram_read_wait_wrap(req: *mut PiClRamReq);

    pub fn pi_cl_ram_write_wait_wrap(req: *mut PiClRamReq);

    pub fn pi_cl_ram_read_wrap(
        device: *mut PiDevice,
        pi_ram_addr: u32,
        addr: *mut cty::c_void,
        size: u32,
        req: *mut PiClRamReq,
    );

    pub fn pi_cl_ram_write_wrap(
        device: *mut PiDevice,
        pi_ram_addr: u32,
        addr: *mut cty::c_void,
        size: u32,
        req: *mut PiClRamReq,
    );

    pub fn abort_all();

    pub fn pi_cl_team_fork_wrap(
        num_cores: usize,
        cluster_fn: extern "C" fn(*mut cty::c_void),
        args: *mut cty::c_void,
    );

    pub fn pi_cl_team_barrier_wrap();

    pub fn pi_l2_malloc(size: cty::c_int) -> *mut cty::c_void;

    pub fn pi_l2_free(chunk: *mut cty::c_void, size: cty::c_int);

    pub fn pi_cl_l1_malloc(cluster: *mut PiDevice, size: cty::c_int) -> *mut cty::c_void;

    pub fn pi_cl_l1_free(cluster: *mut PiDevice, chunk: *mut cty::c_void, size: cty::c_int);

    pub fn rotate_right_wrap(x: cty::c_int, r: cty::c_int) -> cty::c_int;

    pub fn pi_cluster_conf_init(conf: *mut PiClusterConf);

    pub fn pi_open_from_conf(device: *mut PiDevice, conf: *mut cty::c_void);

    pub fn pi_cluster_open(device: *mut PiDevice) -> cty::c_int;

    pub fn print_wrap(str: *const cty::c_char);
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

#[inline(always)]
pub unsafe fn pi_core_id() -> usize {
    let core_id: usize;
    core::arch::asm!("csrr {core_id}, 0x014", core_id = out(reg) core_id,);
    core_id & 0x01f
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
    PiDevicePwmType
}


// Opaque structs
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