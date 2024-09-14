use ash::{vk, Device as AshDevice};
use std::ops::{Deref, DerefMut};

use crate::instance::Instance;

use super::{PhysicalDeviceInfos, EXTENSIONS};

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
    pub fn new(instance: &Instance, infos: PhysicalDeviceInfos) -> RendererDevice {
        // construct queue families infos
        let graphics_info = vk::DeviceQueueCreateInfo::default()
            .queue_family_index(infos.graphics_idx)
            .queue_priorities(&[0.5]);
        let present_info = vk::DeviceQueueCreateInfo::default()
            .queue_family_index(infos.present_idx)
            .queue_priorities(&[0.5]);
        let families_info = if infos.graphics_idx != infos.present_idx {
            vec![graphics_info, present_info]
        } else {
            vec![graphics_info]
        };

        // select extensions
        let extensions = EXTENSIONS;

        // select features (none here)
        let features = vk::PhysicalDeviceFeatures::default();

        // create device info
        let create_info = vk::DeviceCreateInfo::default()
            .enabled_features(&features)
            .enabled_extension_names(&extensions)
            .queue_create_infos(&families_info);

        // instantiate device
        let device = unsafe {
            instance
                .create_device(infos.physical_device, &create_info, None)
                .expect("Failed to create device.")
        };

        RendererDevice {
            device,
            graphics_idx: infos.graphics_idx,
            present_idx: infos.present_idx,
        }
    }

    pub fn swapchain_khr(&self, instance: &Instance) -> ash::khr::swapchain::Device {
        ash::khr::swapchain::Device::new(instance, &self)
    }
}
