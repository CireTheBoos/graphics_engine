use ash::{vk, Entry, Instance};
use std::ffi::CStr;
use winit::raw_window_handle::RawDisplayHandle;

pub struct VkcContext {
    entry: Entry,
    instance: Instance,
}

impl Drop for VkcContext {
    // You must deallocate the tree manually in reverse
    // Deallocate the devices (things that rely on the instance), then the instance, then the entry
    // https://docs.rs/ash/latest/ash/struct.Entry.html#method.create_instance
    fn drop(&mut self) {
        unsafe { self.instance.destroy_instance(None) };
    }
}

impl VkcContext {
    pub fn new(display_handle: RawDisplayHandle) -> VkcContext {
        // load entry and create instance
        let (entry, instance) = instantiate(display_handle);

        // Next : select device and queues

        VkcContext { entry, instance }
    }
}

const LAYERS: [*const i8; 1] = [c"VK_LAYER_KHRONOS_validation".as_ptr()];

// extensions : For using the window
// layers : Khronos validation layer
// Don't fail for now, only panics when failed to load
fn instantiate(display_handle: RawDisplayHandle) -> (Entry, Instance) {
    // load entry
    let entry = unsafe { Entry::load().expect("Failed to load vulkan.") };

    // create basic instance infos
    let application_info = vk::ApplicationInfo::default()
        .api_version(vk::make_api_version(0, 1, 3, 0))
        .application_name(c"Vulkan test");
    let mut create_info = vk::InstanceCreateInfo::default().application_info(&application_info);

    // add extensions
    let enabled_extension_names = ash_window::enumerate_required_extensions(display_handle)
        .expect("Failed to get graphics extensions from display.");
    create_info = create_info.enabled_extension_names(enabled_extension_names);

    // add layers
    if cfg!(debug_assertions) {
        if check_availability(&LAYERS, &entry) {
            create_info = create_info.enabled_layer_names(&LAYERS);
        } else {
            panic!("Some layers are unavailables.");
        }
    }

    // instantiate
    let instance = unsafe {
        entry
            .create_instance(&create_info, None)
            .expect("Failed to create instance.")
    };

    (entry, instance)
}

// checks if each layer is in the available layers of entry
fn check_availability(layers: &[*const i8], entry: &Entry) -> bool {
    let available_layers = unsafe { entry.enumerate_instance_layer_properties().unwrap() };
    for layer in layers {
        let layer = unsafe { CStr::from_ptr(*layer) };
        // Every layer is not available unless it's found
        let mut available = false;
        for available_layer in &available_layers {
            let available_layer = available_layer.layer_name_as_c_str().unwrap();
            if *layer == *available_layer {
                available = true;
                break;
            }
        }
        if !available {
            return false;
        }
    }
    return true;
}
