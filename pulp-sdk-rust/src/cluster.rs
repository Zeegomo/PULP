use core_alloc::boxed::Box;
use crate::*;


const DEFAULT_STACK_SIZE: usize = 1024; //2048;

/// Using raw pointers means we can guarantee at the same time:
/// * no special aliasing since the returned pointer will be used by the C code in ways we cannot predict
/// * pinning
pub struct Cluster<const CORES: usize> {
    device: *mut PiDevice,
    _conf: *mut PiClusterConf,
}

impl<const CORES: usize> Cluster<CORES> {
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
        // let mut cluster_task = PiClusterTask::uninit();
        let allocator = self.l1_allocator();
        let mut cluster_task = Box::leak(Box::new_in(PiClusterTask::uninit(), allocator));
        let exec_fn_args = Box::leak(Box::new_in(ExecFn{f, args} , allocator));
        unsafe {
            print_number(10);
            pi_cluster_task(
                // &mut cluster_task,
                cluster_task,
                Self::execute_inner_pre_fork::<CORES, T>,
                exec_fn_args as *mut _ as *mut cty::c_void,
            );
            print_number(11);
            // cluster_task.set_stack_size(DEFAULT_STACK_SIZE);
            print_number(12);
            print_number(self.device as usize as i32);
            print_number(core::mem::size_of::<PiClusterTask>() as i32);
            // pi_cluster_send_task_to_cl(self.device, &mut cluster_task);
            pi_cluster_send_task_to_cl(self.device, cluster_task);
            print_number(13);
            let _ = Box::from_raw_in(exec_fn_args, allocator);
        }
    }

    extern "C" fn execute_inner_pre_fork<const C: usize, T: Send + Sync>(data: *mut cty::c_void) {
        unsafe {
            //unsafe { if pi_core_id() == 0 { print_number(20); }}
            pi_cl_team_fork(C, Self::execute_inner::<T>, data); 
            //unsafe { if pi_core_id() == 0 { print_number(21); }}
        }
    }

    extern "C" fn execute_inner<T: Send + Sync>(data: *mut cty::c_void) {
        // Safety: we did the allocation ourself, all is good
        //unsafe { if pi_core_id() == 0 { print_number(30); }}
        let ExecFn { ref f, ref args } = unsafe { &*(data as *mut ExecFn<T>) };
        //unsafe { if pi_core_id() == 0 { print_number(31); } }
        f(args)
    }
}

impl<const CORES:usize> Drop for Cluster<CORES> {
    fn drop(&mut self) {
        // TODO
    }
}

struct ExecFn<T> {
    f: extern "C" fn(&T),
    args: T,
}
