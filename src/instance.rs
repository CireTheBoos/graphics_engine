use ash::{
    vk::{self, ApplicationInfo, InstanceCreateInfo, LayerProperties},
    Entry, Instance as AshInstance,
};
use std::{
    ffi::{c_char, CStr},
    ops::Deref,
};
use winit::raw_window_handle::RawDisplayHandle;

const LAYERS: [*const c_char; 1] = [c"VK_LAYER_KHRONOS_validation".as_ptr()];
const _INSTANCE_EXTENSIONS: [*const c_char; 0] = [];

// Wrapper around the ash instance : can provide specific instance types
// (like the one for surfaceKHR or extension's one)
pub struct Instance {
    // entry is stored because it's dynamic/loaded (!= from static/linked)
    // Could be linked, then there would be no entry field
    pub entry: Entry,
    instance: AshInstance,
}

impl Deref for Instance {
    type Target = AshInstance;
    fn deref(&self) -> &Self::Target {
        &self.instance
    }
}

impl Drop for Instance {
    fn drop(&mut self) {
        unsafe { self.destroy_instance(None) };
    }
}

impl Instance {
    pub fn new(display_handle: RawDisplayHandle) -> Instance {
        let entry: Entry = unsafe { Entry::load().expect("Failed to load vulkan.") };
        let instance = create_instance(&entry, display_handle);
        Instance { entry, instance }
    }

    pub fn surface_khr(&self) -> ash::khr::surface::Instance {
        ash::khr::surface::Instance::new(&self.entry, &self.instance)
    }
}

fn create_instance(entry: &Entry, display_handle: RawDisplayHandle) -> AshInstance {
    // create application info
    let application_info = ApplicationInfo::default()
        .api_version(vk::make_api_version(0, 1, 3, 0))
        .application_name(c"Vulkan test");

    // add extensions
    let display_extensions = ash_window::enumerate_required_extensions(display_handle)
        .expect("Failed to get graphics extensions from display.");
    let extensions = [display_extensions, &_INSTANCE_EXTENSIONS].concat();

    // add layers
    let mut layers: Vec<*const c_char> = Vec::new();
    if cfg!(debug_assertions) {
        layers.push(LAYERS[0]);
    }
    let available_layers = unsafe { entry.enumerate_instance_layer_properties() }
        .expect("Failed to get available layers.");
    for layer in &layers {
        if !is_layer_available(*layer, &available_layers) {
            panic!("Some layers are unavailable.");
        }
    }

    // create instance create info
    let create_info = InstanceCreateInfo::default()
        .application_info(&application_info)
        .enabled_extension_names(&extensions)
        .enabled_layer_names(&layers);

    // instantiate
    unsafe {
        entry
            .create_instance(&create_info, None)
            .expect("Failed to create instance.")
    }
}

fn is_layer_available(layer: *const c_char, available_layers: &Vec<LayerProperties>) -> bool {
    let layer = unsafe { CStr::from_ptr(layer) };
    available_layers
        .iter()
        .any(|available_layer| *available_layer.layer_name_as_c_str().unwrap() == *layer)
}
