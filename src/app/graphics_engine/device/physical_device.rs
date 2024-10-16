use ash::vk::{
    ColorSpaceKHR, ExtensionProperties, Format, PhysicalDevice, PhysicalDeviceType, PresentModeKHR,
    QueueFlags, SurfaceCapabilitiesKHR, SurfaceFormatKHR, SurfaceKHR,
};
use std::ffi::{c_char, CStr};

use super::SWAPCHAIN_KHR_EXTENSION;
use crate::app::instance::Instance;

#[derive(Debug)]
pub struct PhysicalDeviceInfos {
    pub physical_device: PhysicalDevice,
    score: u32,
    pub present_idx: u32,
    pub graphics_idx: u32,
    pub transfer_idx: u32,
    pub capabilities: SurfaceCapabilitiesKHR,
    pub surface_format: SurfaceFormatKHR,
    pub present_mode: PresentModeKHR,
}

// returns most appropriate physical device along with its infos
pub fn select_physical_device(
    instance: &Instance,
    surface: &SurfaceKHR,
) -> Result<PhysicalDeviceInfos, ()> {
    // Query all physical devices
    let physical_devices: Vec<PhysicalDeviceInfos> =
        unsafe { instance.enumerate_physical_devices() }
            .expect("Failed to query physical devices.")
            .into_iter()
            .filter_map(|physical_device| {
                query_physical_device_infos(instance, surface, physical_device).ok()
            })
            .collect();

    // Select highest scoring device
    physical_devices
        .into_iter()
        .max_by_key(|device| device.score)
        .ok_or(())
}

// Query infos for a physical device (fails when the device is unsuitable)
fn query_physical_device_infos(
    instance: &Instance,
    surface: &SurfaceKHR,
    physical_device: PhysicalDevice,
) -> Result<PhysicalDeviceInfos, ()> {
    // fetching general device data
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

    // transfer queue (!= graphics queue)
    let transfer_idx = queue_families
        .iter()
        .position(|queue_family| {
            queue_family.queue_flags.contains(QueueFlags::TRANSFER)
                && !queue_family.queue_flags.contains(QueueFlags::GRAPHICS)
        })
        .ok_or(())? as u32;

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

    // swapchain extension support
    if !is_extension_available(SWAPCHAIN_KHR_EXTENSION, &available_extensions) {
        return Err(());
    }

    // fetching surface capabilities, format, present_mode for this device
    let capabilities = unsafe {
        instance
            .surface_khr()
            .get_physical_device_surface_capabilities(physical_device, *surface)
    }
    .unwrap();
    let available_surface_formats = unsafe {
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

    // SCORING
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
    let (surface_format, surface_format_score) = choose_best_format(&available_surface_formats);
    score += surface_format_score;

    // FIFO present mode is prefered
    let (present_mode, present_mode_score) = choose_best_present_mode(&available_present_modes);
    score += present_mode_score;

    Ok(PhysicalDeviceInfos {
        physical_device,
        score,
        capabilities,
        surface_format,
        present_mode,
        graphics_idx,
        present_idx,
        transfer_idx,
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
    available_formats
        .iter()
        .enumerate()
        // filter available surface formats based on match : pattern => score
        .filter_map(|(idx, format)| {
            if idx == 0 {
                return Some((*format, 0));
            }
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
    available_present_modes
        .iter()
        .enumerate()
        // filter available present modes based on match : pattern => score
        .filter_map(|(idx, present_mode)| {
            if idx == 0 {
                return Some((*present_mode, 0));
            }
            match *present_mode {
                PresentModeKHR::FIFO => Some((*present_mode, 10)),
                _ => None,
            }
        })
        .max_by_key(|(_, score)| *score)
        .unwrap()
}
