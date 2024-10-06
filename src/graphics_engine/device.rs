mod allocator;
mod physical_device;

use crate::instance::Instance;

pub use allocator::CustomBuffer;

use ash::vk::{self, SurfaceKHR};
use physical_device::PhysicalDeviceInfos;
use std::{ffi::c_char, ops::Deref};

const SWAPCHAIN_KHR_EXTENSION: *const c_char = c"VK_KHR_swapchain".as_ptr();

// Custom device for rendering :
// - swapchainKHR extension + support for presenting on "surface"
// - Hold infos about the physical device in use and the surface
// - Hold a VMA instance
pub struct Device {
    device: ash::Device,
    // Option bc allocator must drop before device destruction
    allocator: Option<vk_mem::Allocator>,
    // swapchainKHR extension fns
    swapchain_khr_device: ash::khr::swapchain::Device,
    pub infos: PhysicalDeviceInfos,
}

// Deref : ash::Device
impl Deref for Device {
    type Target = ash::Device;
    fn deref(&self) -> &Self::Target {
        &self.device
    }
}

// Drop : Drop allocator then destroy device
impl Drop for Device {
    fn drop(&mut self) {
        unsafe {
            self.allocator = None;
            self.destroy_device(None);
        }
    }
}

impl Device {
    pub fn new(instance: &Instance, surface: &SurfaceKHR) -> Device {
        let infos = physical_device::select_physical_device(instance, surface)
            .expect("Failed to find a suitable physical device.");
        let device = create_device(instance, &infos);
        let allocator = Some(allocator::create_allocator(
            instance,
            &device,
            infos.physical_device,
        ));
        let swapchain_khr_device = ash::khr::swapchain::Device::new(instance, &device);
        Device {
            device,
            allocator,
            swapchain_khr_device,
            infos,
        }
    }

    pub fn swapchain_khr(&self) -> &ash::khr::swapchain::Device {
        &self.swapchain_khr_device
    }

    pub fn allocator(&self) -> &vk_mem::Allocator {
        // allocator's option is None only when dropping
        self.allocator.as_ref().unwrap()
    }
}

fn create_device(instance: &Instance, infos: &PhysicalDeviceInfos) -> ash::Device {
    // SPECIFY : queues requested for each queue family
    let graphics_queues_info = vk::DeviceQueueCreateInfo::default()
        .queue_family_index(infos.graphics_idx)
        .queue_priorities(&[0.5]);
    let present_queues_info = vk::DeviceQueueCreateInfo::default()
        .queue_family_index(infos.present_idx)
        .queue_priorities(&[0.5]);
    let transfer_queues_info = vk::DeviceQueueCreateInfo::default()
        .queue_family_index(infos.transfer_idx)
        .queue_priorities(&[0.5]);
    let mut queue_create_infos = vec![
        graphics_queues_info,
        present_queues_info,
        transfer_queues_info,
    ];
    // removes duplicates
    queue_create_infos.sort_by_key(|info| info.queue_family_index);
    queue_create_infos.dedup_by_key(|info| info.queue_family_index);

    // SPECIFY : extensions
    let swapchain_extension = vec![SWAPCHAIN_KHR_EXTENSION];
    let extensions = [swapchain_extension].concat();

    // CREATE : device
    let create_info = vk::DeviceCreateInfo::default()
        .queue_create_infos(&queue_create_infos)
        .enabled_extension_names(&extensions);
    unsafe { instance.create_device(infos.physical_device, &create_info, None) }
        .expect("Failed to create device.")
}
