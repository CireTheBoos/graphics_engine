use ash::{
    vk::{self, ApplicationInfo, ExtensionProperties, InstanceCreateInfo, LayerProperties},
    Entry,
};
use std::{
    ffi::{c_char, CStr},
    ops::Deref,
};
use winit::raw_window_handle::RawDisplayHandle;

const VALIDATION_LAYER: *const c_char = c"VK_LAYER_KHRONOS_validation".as_ptr();

// Custom instance for presenting :
// - Appropriate extensions for creating surfaces on the given display => surfaceKHR extension + OS-specific window extension
// - Hold entry => Must be the only instance
// - Validation layers on Debug
pub struct Instance {
    entry: Entry,
    instance: ash::Instance,
    // surfaceKHR extension vk fns
    surface_khr_instance: ash::khr::surface::Instance,
}

// Deref : ash::Instance
impl Deref for Instance {
    type Target = ash::Instance;
    fn deref(&self) -> &Self::Target {
        &self.instance
    }
}

// Drop : Destroy instance
impl Drop for Instance {
    fn drop(&mut self) {
        unsafe { self.destroy_instance(None) };
    }
}

impl Instance {
    // "raw_display_handle" arg used to enable display_compatible surfaceKHR extension
    pub fn new(raw_display_handle: RawDisplayHandle) -> Instance {
        let entry: Entry = unsafe { Entry::load().expect("Failed to load vulkan.") };
        let instance = create_instance(&entry, raw_display_handle);
        let surface_khr_instance = ash::khr::surface::Instance::new(&entry, &instance);
        Instance {
            entry,
            instance,
            surface_khr_instance,
        }
    }

    pub fn entry(&self) -> &Entry {
        &self.entry
    }

    pub fn surface_khr(&self) -> &ash::khr::surface::Instance {
        &self.surface_khr_instance
    }
}

fn create_instance(entry: &Entry, raw_display_handle: RawDisplayHandle) -> ash::Instance {
    // SPECIFY : layers
    let validation_layer = if cfg!(debug_assertions) {
        vec![VALIDATION_LAYER]
    } else {
        Vec::new()
    };
    let layers = [validation_layer].concat();
    // availability check (panic if unavailables)
    let available_layers = unsafe { entry.enumerate_instance_layer_properties() }
        .expect("Failed to get available layers.");
    for layer in &layers {
        if !is_layer_available(*layer, &available_layers) {
            panic!("Some layers are unavailable.");
        }
    }

    // SPECIFY : extensions
    let surface_extensions = ash_window::enumerate_required_extensions(raw_display_handle)
        .expect("Failed to get graphics extensions from display.")
        .to_vec();
    let extensions = [surface_extensions].concat();
    // availability check (panic if unavailables)
    let available_extensions = unsafe { entry.enumerate_instance_extension_properties(None) }
        .expect("Failed to get available extensions.");
    for extension in &extensions {
        if !is_extension_available(*extension, &available_extensions) {
            panic!("Some extensions are not supported.")
        }
    }

    // SPECIFY : application info
    let application_info = ApplicationInfo::default().api_version(vk::make_api_version(0, 1, 3, 0));

    // CREATE : instance
    let create_info = InstanceCreateInfo::default()
        .enabled_layer_names(&layers)
        .enabled_extension_names(&extensions)
        .application_info(&application_info);
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

fn is_extension_available(
    extension: *const c_char,
    available_extensions: &Vec<ExtensionProperties>,
) -> bool {
    let extension = unsafe { CStr::from_ptr(extension) };
    available_extensions.iter().any(|available_extension| {
        *available_extension.extension_name_as_c_str().unwrap() == *extension
    })
}
