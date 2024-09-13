use ash::vk::SurfaceKHR;
use ash::{
    vk::{self, PhysicalDevice, QueueFlags},
    Device as AshDevice,
};
use std::ops::{Deref, DerefMut};

use crate::vk_loader::Loader;

// Just a wrapper around ash device that holds families idx
// Only for device with graphics
pub struct RendererDevice {
    device: AshDevice,
    pub graphics_idx: u32,
    pub present_idx: u32,
}

impl Deref for RendererDevice {
    type Target = AshDevice;
    fn deref(&self) -> &Self::Target {
        &self.device
    }
}
impl DerefMut for RendererDevice {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.device
    }
}

impl RendererDevice {
    pub fn new(
        instance: &Loader,
        physical_device: &PhysicalDevice,
        surface: &SurfaceKHR,
    ) -> RendererDevice {
        let queue_families =
            unsafe { instance.get_physical_device_queue_family_properties(*physical_device) };

        // get the graphics family idx
        let graphics_idx = queue_families
            .iter()
            .position(|family| family.queue_flags.contains(QueueFlags::GRAPHICS))
            .unwrap() as u32; // Existence checked when selecting device

        // get the presentation family idx
        let present_idx = queue_families
            .iter()
            .enumerate()
            .position(|(idx, _)| unsafe {
                instance
                    .surface_khr()
                    .get_physical_device_surface_support(*physical_device, idx as u32, *surface)
                    .unwrap()
            })
            .unwrap() as u32;

        // construct queue families infos (just one queue on graphics here)
        let graphics_info = vk::DeviceQueueCreateInfo::default()
            .queue_family_index(graphics_idx)
            .queue_priorities(&[0.5]);
        let present_info = vk::DeviceQueueCreateInfo::default()
            .queue_family_index(present_idx)
            .queue_priorities(&[0.5]);
        let families_info = if graphics_idx != present_idx {
            vec![graphics_info, present_info]
        } else {
            vec![graphics_info]
        };

        // select features (none here)
        let features = vk::PhysicalDeviceFeatures::default();

        // create device info
        let create_info = vk::DeviceCreateInfo::default()
            .enabled_features(&features)
            .queue_create_infos(&families_info);

        // instantiate device
        let device = unsafe {
            instance
                .create_device(*physical_device, &create_info, None)
                .expect("Failed to create device.")
        };
        RendererDevice {
            device,
            graphics_idx,
            present_idx,
        }
    }
}
