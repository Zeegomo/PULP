use alloc::boxed::Box;
use pulp_sdk_rust::*;

/// Using raw pointers means we can guarantee at the same time:
/// * no special aliasing since the returned pointer will be used by the C code in ways we cannot predict
/// * pinning
pub struct Cluster {
    device: *mut PiDevice,
    _conf: *mut PiClusterConf,
}

impl Cluster {
    pub fn new() -> Result<Self, ()> {
        let device: *mut _ = Box::leak(Box::new_in(PiDevice::uninit(), L2Allocator));
        let _conf: *mut _ = Box::leak(Box::new_in(PiClusterConf::uninit(), L2Allocator));

        unsafe {
            pi_cluster_conf_init(_conf);
            pi_open_from_conf(device, _conf as *mut cty::c_void);
            if pi_cluster_open(device as *mut PiDevice) != 0 {
                return Err(());
            }

            Ok(Self { device, _conf })
        }
    }

    /// Returns an allocator that uses the cluster L1 memory
    pub fn l1_allocator(&self) -> ClusterAllocator {
        ClusterAllocator::new(self.device)
    }

    /// Schedule a function for execution on each cluster core.
    /// This is a blocking function.
    pub fn execute_fn_parallel<T: Send + Sync>(&mut self, f: extern "C" fn(&T), args: T) {
        let mut cluster_task = PiClusterTask::uninit();
        let allocator = self.l1_allocator();
        let exec_fn_args = Box::leak(Box::new_in(ExecFn{f, args} , allocator));
        unsafe {
            pi_cluster_task(
                &mut cluster_task,
                Self::execute_inner_pre_fork::<8, T>,
                exec_fn_args as *mut _ as *mut cty::c_void,
            );
            pi_cluster_send_task_to_cl(self.device, &mut cluster_task);
            let _ = Box::from_raw_in(exec_fn_args, allocator);
        }
    }

    extern "C" fn execute_inner_pre_fork<const CORES: usize, T: Send + Sync>(data: *mut cty::c_void) {
        unsafe { pi_cl_team_fork(CORES, Self::execute_inner::<T>, data) }
    }

    extern "C" fn execute_inner<T: Send + Sync>(data: *mut cty::c_void) {
        // Safety: we did the allocation ourself, all is good
        let ExecFn { ref f, ref args } = unsafe { &*(data as *mut ExecFn<T>) };
        f(args)
    }
}

impl Drop for Cluster {
    fn drop(&mut self) {
        // TODO
    }
}

struct ExecFn<T> {
    f: extern "C" fn(&T),
    args: T,
}
