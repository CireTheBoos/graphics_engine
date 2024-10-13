use ash::vk::{Fence, FenceCreateFlags, FenceCreateInfo, Semaphore, SemaphoreCreateInfo};

use super::Device;

impl Device {
    pub fn new_semaphore(&self) -> Semaphore {
        let semaphore_create_info = SemaphoreCreateInfo::default();
        unsafe { self.create_semaphore(&semaphore_create_info, None) }
            .expect("Failed to create semaphore.")
    }

    pub fn new_fence(&self, signaled: bool) -> Fence {
        let fence_create_info = if signaled {
            FenceCreateInfo::default().flags(FenceCreateFlags::SIGNALED)
        } else {
            FenceCreateInfo::default()
        };
        unsafe { self.create_fence(&fence_create_info, None) }.expect("Failed to create fence.")
    }

    pub fn wait_reset_fence(&self, fence: Fence, timeout: Option<u64>) {
        let fences = [fence];
        self.wait_reset_fences(&fences, true, timeout);
    }

    fn wait_reset_fences(&self, fences: &[Fence], wait_all: bool, timeout: Option<u64>) {
        let timeout = timeout.unwrap_or(u64::MAX);
        unsafe {
            self.wait_for_fences(fences, wait_all, timeout)
                .expect("Failed to wait for the fences.");
            self.reset_fences(fences)
                .expect("Failed to reset the fences.");
        }
    }
}
