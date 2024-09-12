mod device;

use ash::{
    vk::{self, PhysicalDevice, PhysicalDeviceType, Queue, QueueFlags},
    Entry, Instance,
};
use device::Device;
use std::ffi::{c_char, CStr};
use winit::raw_window_handle::RawDisplayHandle;

const LAYERS: [*const c_char; 1] = [c"VK_LAYER_KHRONOS_validation".as_ptr()];

// entry is stored because it's dynamic/loaded (!= from static/linked).
pub struct Context {
    // loaders
    pub _entry: Entry,
    pub instance: Instance,
    pub device: Device,
    // queues
    pub graphics: Queue,
}

impl Drop for Context {
    // You must deallocate the loaders manually in reverse
    // Devices (rely on instance), then Instance (rely on entry), then Entry
    // https://docs.rs/ash/latest/ash/struct.Entry.html#method.create_instance
    fn drop(&mut self) {
        unsafe {
            self.device.destroy_device(None);
            self.instance.destroy_instance(None);
        }
    }
}

impl Context {
    pub fn new(display_handle: RawDisplayHandle) -> Context {
        // load entry and create instance
        let (_entry, instance) = instantiate(display_handle);
        // select device
        let device = select_device(&instance);
        // Get queues
        let graphics = unsafe { device.get_device_queue(device.graphics_idx, 0) };

        Context {
            _entry,
            instance,
            device,
            graphics,
        }
    }
}

fn select_device(instance: &Instance) -> Device {
    // enumerate physical devices
    let physical_device_list = unsafe {
        instance
            .enumerate_physical_devices()
            .expect("Failed to query physical devices.")
    };

    // select one that suits our needs
    let chosen_one = physical_device_list
        .iter()
        .max_by_key(|physical_device| score_suitability(instance, physical_device))
        .expect("No physical devices implement Vulkan.");
    if score_suitability(instance, chosen_one) == 0 {
        panic!("No suitable device found.");
    }

    // construct device
    Device::new(instance, chosen_one)
}

// 0 means unsuitable
fn score_suitability(instance: &Instance, physical_device: &PhysicalDevice) -> u32 {
    let mut score = 0;

    // graphic queue check : early return 0 if no queue for graphics
    let queue_families =
        unsafe { instance.get_physical_device_queue_family_properties(*physical_device) };

    let can_do_graphics = queue_families
        .iter()
        .any(|queue_family| queue_family.queue_flags.contains(QueueFlags::GRAPHICS));
    if !can_do_graphics {
        return 0;
    }

    // properties scoring : dedicated gpu are prefered
    let properties = unsafe { instance.get_physical_device_properties(*physical_device) };
    match properties.device_type {
        PhysicalDeviceType::DISCRETE_GPU | PhysicalDeviceType::VIRTUAL_GPU => {
            score += 10;
        }
        PhysicalDeviceType::INTEGRATED_GPU => {
            score += 5;
        }
        _ => {
            score += 1;
        }
    }

    score
}

fn instantiate(display_handle: RawDisplayHandle) -> (Entry, Instance) {
    // load entry
    let entry: Entry = unsafe { Entry::load().expect("Failed to load vulkan.") };

    // create application info
    let application_info = vk::ApplicationInfo::default()
        .api_version(vk::make_api_version(0, 1, 3, 0))
        .application_name(c"Vulkan test");

    // select extensions
    let extensions = ash_window::enumerate_required_extensions(display_handle)
        .expect("Failed to get graphics extensions from display.");

    // select layers
    let mut layers: Vec<*const c_char> = Vec::new();
    if cfg!(debug_assertions) {
        if layer_available(LAYERS[0], &entry) {
            layers.push(LAYERS[0]);
        } else {
            panic!("Some layers are unavailables.");
        }
    }

    // create instance create info
    let create_info = vk::InstanceCreateInfo::default()
        .application_info(&application_info)
        .enabled_extension_names(extensions)
        .enabled_layer_names(&layers);

    // instantiate
    let instance = unsafe {
        entry
            .create_instance(&create_info, None)
            .expect("Failed to create instance.")
    };

    (entry, instance)
}

fn layer_available(layer: *const c_char, entry: &Entry) -> bool {
    let layer = unsafe { CStr::from_ptr(layer) };

    // get available layers from entry
    let available_layers = unsafe { entry.enumerate_instance_layer_properties().unwrap() };
    available_layers
        .iter()
        .any(|available_layer| *available_layer.layer_name_as_c_str().unwrap() == *layer)
}
