use crate::instance::Instance;

use ash::{
    vk::{
        self, ColorSpaceKHR, ExtensionProperties, Extent2D, Format, PhysicalDevice,
        PhysicalDeviceType, PresentModeKHR, QueueFlags, SurfaceCapabilitiesKHR, SurfaceFormatKHR,
        SurfaceKHR,
    },
    Device as AshDevice,
};
use std::{
    ffi::{c_char, CStr},
    ops::Deref,
    ptr::NonNull,
};

const DEVICE_EXTENSIONS: [*const c_char; 1] = [c"VK_KHR_swapchain".as_ptr()];

// Wrapper around ash::Device : can provide device-level extension vk fns
pub struct RendererDevice {
    instance: NonNull<Instance>,
    device: AshDevice,
    pub graphics_idx: u32,
    pub present_idx: u32,
}

// Deref to ash::Device
impl Deref for RendererDevice {
    type Target = AshDevice;
    fn deref(&self) -> &Self::Target {
        &self.device
    }
}

// Destroy device
impl Drop for RendererDevice {
    fn drop(&mut self) {
        unsafe { self.destroy_device(None) };
    }
}

impl RendererDevice {
    pub fn new(instance: &Instance, surface: &SurfaceKHR) -> (RendererDevice, PhysicalDeviceInfos) {
        let physical_device_infos = select_physical_device(instance, surface);
        let device = create_device(instance, physical_device_infos);
        let device = RendererDevice {
            instance: NonNull::from(instance),
            device,
            graphics_idx: physical_device_infos.graphics_idx,
            present_idx: physical_device_infos.present_idx,
        };
        (device, physical_device_infos)
    }

    pub fn swapchain_khr(&self) -> ash::khr::swapchain::Device {
        ash::khr::swapchain::Device::new(unsafe { self.instance.as_ref() }, &self)
    }
}

fn create_device(instance: &Instance, infos: PhysicalDeviceInfos) -> AshDevice {
    // SPECIFY : queue families
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

    // SPECIFY : extensions
    let extensions = DEVICE_EXTENSIONS;

    // SPECIFY : features (none here)
    let features = vk::PhysicalDeviceFeatures::default();

    // CREATE : device
    let create_info = vk::DeviceCreateInfo::default()
        .queue_create_infos(&families_info)
        .enabled_extension_names(&extensions)
        .enabled_features(&features);
    unsafe {
        instance
            .create_device(infos.physical_device, &create_info, None)
            .expect("Failed to create device.")
    }
}

#[derive(Clone, Copy)]
pub struct PhysicalDeviceInfos {
    physical_device: PhysicalDevice,
    score: u32,
    pub capabilities: SurfaceCapabilitiesKHR,
    pub extent: Extent2D,
    pub format: SurfaceFormatKHR,
    pub present_mode: PresentModeKHR,
    pub graphics_idx: u32,
    pub present_idx: u32,
}

fn select_physical_device(instance: &Instance, surface: &SurfaceKHR) -> PhysicalDeviceInfos {
    // query all physical devices
    let physical_devices: Vec<PhysicalDeviceInfos> =
        unsafe { instance.enumerate_physical_devices() }
            .expect("Failed to query physical devices.")
            .into_iter()
            .filter_map(|physical_device| {
                query_physical_device_infos(instance, surface, physical_device).ok()
            })
            .collect();

    // panic if no suitable device
    if physical_devices.is_empty() {
        panic!("No suitable device found.");
    }

    // select the highest scoring device
    physical_devices
        .into_iter()
        .max_by_key(|device| device.score)
        .unwrap()
}

// Query infos for a physical device (fails when the device is unsuitable)
fn query_physical_device_infos(
    instance: &Instance,
    surface: &SurfaceKHR,
    physical_device: PhysicalDevice,
) -> Result<PhysicalDeviceInfos, ()> {
    // device data
    let queue_families =
        unsafe { instance.get_physical_device_queue_family_properties(physical_device) };
    let available_extensions =
        unsafe { instance.enumerate_device_extension_properties(physical_device) }
            .expect("Failed to get device extensions.");
    let properties = unsafe { instance.get_physical_device_properties(physical_device) };

    // REQUIRED

    // graphics queue presence
    let graphics_idx = queue_families
        .iter()
        .position(|queue_family| queue_family.queue_flags.contains(QueueFlags::GRAPHICS))
        .ok_or(())? as u32; // Convert Option to Result : Ok for Some and Err for None

    // surface support
    let present_idx = queue_families
        .iter()
        .enumerate()
        .position(|(idx, _)| unsafe {
            instance
                .surface_khr()
                .get_physical_device_surface_support(physical_device, idx as u32, *surface)
                .unwrap()
        })
        .ok_or(())? as u32;

    // swapchain support
    let has_swapchain_extension =
        is_extension_available(DEVICE_EXTENSIONS[0], &available_extensions);
    if !has_swapchain_extension {
        return Err(());
    }

    // fetching swapchain capabilities, format, present_mode
    // unwrap()s shouldn't fail because swapchain is supported
    let capabilities = unsafe {
        instance
            .surface_khr()
            .get_physical_device_surface_capabilities(physical_device, *surface)
    }
    .unwrap();
    let available_formats = unsafe {
        instance
            .surface_khr()
            .get_physical_device_surface_formats(physical_device, *surface)
    }
    .unwrap();
    let available_present_modes = unsafe {
        instance
            .surface_khr()
            .get_physical_device_surface_present_modes(physical_device, *surface)
    }
    .unwrap();
    if available_formats.is_empty() || available_present_modes.is_empty() {
        return Err(());
    }

    // SCORE
    let mut score = 0;

    // dedicated gpu are prefered
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

    // SRGB_8 format is prefered
    let (format, format_score) = choose_best_format(&available_formats);
    score += format_score;

    // FIFO present mode is prefered
    let (present_mode, present_mode_score) = choose_best_present_mode(&available_present_modes);
    score += present_mode_score;

    let extent = choose_best_extent(&capabilities);

    Ok(PhysicalDeviceInfos {
        physical_device,
        score,
        capabilities,
        extent,
        format,
        present_mode,
        graphics_idx,
        present_idx,
    })
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

fn choose_best_format(available_formats: &Vec<SurfaceFormatKHR>) -> (SurfaceFormatKHR, u32) {
    // filter available formats based on match : pattern => score
    // keep the first one in case no format match
    // then pick the highest score
    available_formats
        .iter()
        .enumerate()
        .filter_map(|(idx, format)| {
            if idx == 0 {
                return Some((*format, 0));
            }
            // Here's the scoring
            match (format.format, format.color_space) {
                (Format::B8G8R8A8_SRGB, ColorSpaceKHR::SRGB_NONLINEAR) => Some((*format, 10)),
                _ => None,
            }
        })
        .max_by_key(|(_, score)| *score)
        .unwrap()
}

fn choose_best_present_mode(
    available_present_modes: &Vec<PresentModeKHR>,
) -> (PresentModeKHR, u32) {
    // filter available present modes based on match : pattern => score
    // keep the first one in case no present mode match
    // then pick the highest score
    available_present_modes
        .iter()
        .enumerate()
        .filter_map(|(idx, present_mode)| {
            if idx == 0 {
                return Some((*present_mode, 0));
            }
            // Here's the scoring
            match *present_mode {
                PresentModeKHR::FIFO => Some((*present_mode, 10)),
                _ => None,
            }
        })
        .max_by_key(|(_, score)| *score)
        .unwrap()
}

fn choose_best_extent(capabilities: &SurfaceCapabilitiesKHR) -> Extent2D {
    capabilities.current_extent
}
