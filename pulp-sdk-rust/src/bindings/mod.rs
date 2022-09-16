mod types;
pub use types::*;

extern "C" {
    pub fn pi_cl_dma_cmd_wrap(
        ext: cty::uint32_t,
        loc: cty::uint32_t,
        size: cty::uint32_t,
        dir: PiClDmaDirE,
        cmd: *mut PiClDmaCmd,
    );

    pub fn pi_cl_dma_wait_wrap(copy: *mut cty::c_void);

    // pub fn pi_cl_ram_read_wait_wrap(req: *mut PiClRamReq);

    // pub fn pi_cl_ram_write_wait_wrap(req: *mut PiClRamReq);

    // pub fn pi_cl_ram_read_wrap(
    //     device: *mut PiDevice,
    //     pi_ram_addr: u32,
    //     addr: *mut cty::c_void,
    //     size: u32,
    //     req: *mut PiClRamReq,
    // );

    // pub fn pi_cl_ram_write_wrap(
    //     device: *mut PiDevice,
    //     pi_ram_addr: u32,
    //     addr: *mut cty::c_void,
    //     size: u32,
    //     req: *mut PiClRamReq,
    // );

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

    // pub fn pi_cluster_task_wrap(
    //     task: *mut PiClusterTask,
    //     entry: extern "C" fn(arg: *mut cty::c_void),
    //     arg: *mut cty::c_void,
    // ) -> *mut PiClusterTask;

    // pub fn pi_cluster_send_task_to_cl(device: *mut PiDevice, task: *mut PiClusterTask) -> cty::c_int;
}

// pub unsafe fn pi_cluster_task(task: *mut PiClusterTask,
//     entry: extern "C" fn(arg: *mut cty::c_void),
//     arg: *mut cty::c_void,
// ) -> *mut PiClusterTask {
//     pi_cluster_task_wrap(task, entry, arg)
// }

#[inline(always)]
pub unsafe fn pi_core_id() -> usize {
    let core_id: usize;
    core::arch::asm!("csrr {core_id}, 0x014", core_id = out(reg) core_id,);
    core_id & 0x01f
}

pub unsafe fn pi_cl_dma_cmd(
    ext: *mut u8,
    loc: *mut u8,
    size: usize,
    dir: PiClDmaDirE,
    cmd: &mut PiClDmaCmd,
) {
    pi_cl_dma_cmd_wrap(
        ext as usize as u32,
        loc as usize as u32,
        size as u32,
        dir,
        cmd as *mut PiClDmaCmd,
    )
}

pub fn pi_cl_dma_wait(copy: &mut PiClDmaCmd) {
    unsafe { pi_cl_dma_wait_wrap(copy as *mut PiClDmaCmd as *mut cty::c_void) }
}

// pub fn pi_cl_ram_read_wait(req: &mut PiClRamReq) {
//     unsafe { pi_cl_ram_read_wait_wrap(req as *mut PiClRamReq) }
// }

// pub fn pi_cl_ram_write_wait(req: &mut PiClRamReq) {
//     unsafe { pi_cl_ram_write_wait_wrap(req as *mut PiClRamReq) }
// // }

// pub unsafe fn pi_cl_ram_read(
//     device: *mut PiDevice,
//     pi_ram_addr: *mut u8,
//     addr: *mut u8,
//     size: usize,
//     req: &mut PiClRamReq,
// ) {
//     pi_cl_ram_read_wrap(
//         device,
//         pi_ram_addr as cty::uint32_t,
//         addr as *mut cty::c_void,
//         size as cty::uint32_t,
//         req as *mut PiClRamReq,
//     )
// }

// pub unsafe fn pi_cl_ram_write(
//     device: *mut PiDevice,
//     pi_ram_addr: *mut u8,
//     addr: *mut u8,
//     size: usize,
//     req: &mut PiClRamReq,
// ) {
//     pi_cl_ram_write_wrap(
//         device,
//         pi_ram_addr as cty::uint32_t,
//         addr as *mut cty::c_void,
//         size as cty::uint32_t,
//         req as *mut PiClRamReq,
//     )
// }
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
