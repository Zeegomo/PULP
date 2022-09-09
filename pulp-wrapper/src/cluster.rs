use alloc::boxed::Box;
use core::pin::Pin;
use pulp_sdk_rust::{
    pi_cluster_conf_init, pi_cluster_open, pi_open_from_conf, ClusterAllocator, L2Allocator,
    PiClusterConf, PiDevice,
};

/// Using raw pointers means we can guarantee at the same time:
/// * no special aliasing since the returned pointer will be used by the C code in ways we cannot predict
/// * pinning
pub struct Cluster {
    device: *mut PiDevice,
    conf: *mut PiClusterConf,
}

impl Cluster {
    pub fn new() -> Result<Self, ()> {
        let device: *mut _ = Box::leak(Box::new_in(PiDevice::uninit(), L2Allocator));
        let conf: *mut _ = Box::leak(Box::new_in(PiClusterConf::uninit(), L2Allocator));

        unsafe {
            pi_cluster_conf_init(conf);
            pi_open_from_conf(device, conf as *mut cty::c_void);
            if pi_cluster_open(device as *mut PiDevice) != 0 {
                return Err(());
            }

            Ok(Self { device, conf })
        }
    }

    // This should really be used only by C libraries
    pub fn as_mut_ptr(&mut self) -> *mut PiDevice {
        self.device
    }

    pub fn l1_allocator(&self) -> ClusterAllocator {
        ClusterAllocator::new(self.device)
    }
}

impl Drop for Cluster {
    fn drop(&mut self) {
        // TODO
    }
}
