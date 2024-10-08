use ash::{
    vk::{Fence, FenceCreateFlags, FenceCreateInfo, Semaphore, SemaphoreCreateInfo},
    Device,
};

pub fn new_semaphore(device: &Device) -> Semaphore {
    let semaphore_create_info = SemaphoreCreateInfo::default();
    unsafe { device.create_semaphore(&semaphore_create_info, None) }
        .expect("Failed to create semaphore.")
}

pub fn new_fence(device: &Device, signaled: bool) -> Fence {
    let fence_create_info = if signaled {
        FenceCreateInfo::default().flags(FenceCreateFlags::SIGNALED)
    } else {
        FenceCreateInfo::default()
    };
    unsafe { device.create_fence(&fence_create_info, None) }.expect("Failed to create fence.")
}

fn wait_reset_fences(device: &Device, fences: &[Fence], wait_all: bool, timeout: Option<u64>) {
    let timeout = timeout.unwrap_or(u64::MAX);
    unsafe {
        device
            .wait_for_fences(fences, wait_all, timeout)
            .expect("Failed to wait for the fences.");
        device
            .reset_fences(fences)
            .expect("Failed to reset the fences.");
    }
}

pub fn wait_reset_fence(device: &Device, fence: Fence, timeout: Option<u64>) {
    let fences = [fence];
    wait_reset_fences(device, &fences, true, timeout);
}
