use ash::{
    vk::{self, PhysicalDevice, QueueFlags},
    Device as AshDevice, Instance,
};
use std::ops::{Deref, DerefMut};

// Just a wrapper around ash device that holds families idx
// Only for device with graphics
pub struct Device {
    device: AshDevice,
    pub graphics_idx: u32,
}

impl Deref for Device {
    type Target = AshDevice;
    fn deref(&self) -> &Self::Target {
        &self.device
    }
}
impl DerefMut for Device {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.device
    }
}

impl Device {
    pub fn new(instance: &Instance, physical_device: &PhysicalDevice) -> Device {
        // get the graphics family idx
        let queue_families =
            unsafe { instance.get_physical_device_queue_family_properties(*physical_device) };
        let graphics_idx = queue_families
            .iter()
            .position(|family| family.queue_flags.contains(QueueFlags::GRAPHICS))
            .unwrap() as u32; // Existence checked when selecting device

        // construct queue families infos (just one queue on graphics here)
        let graphics_info = vk::DeviceQueueCreateInfo::default()
            .queue_family_index(graphics_idx)
            .queue_priorities(&[1.0]);
        let families_info = [graphics_info];

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
        Device {
            device,
            graphics_idx,
        }
    }
}
