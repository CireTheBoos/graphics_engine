mod device;

use std::{
    ffi::{c_char, CStr},
    ptr::NonNull,
};

use crate::instance::Instance;
use ash::vk::{
    ExtensionProperties, PhysicalDevice, PhysicalDeviceType, PresentModeKHR, Queue, QueueFlags,
    SurfaceCapabilitiesKHR, SurfaceFormatKHR, SurfaceKHR,
};
use device::RendererDevice;
use winit::{
    dpi::PhysicalSize,
    event_loop::ActiveEventLoop,
    raw_window_handle::{HasDisplayHandle, HasWindowHandle},
    window::Window,
};

const WIDTH: u32 = 800;
const HEIGHT: u32 = 800;

const EXTENSIONS: [*const c_char; 1] = [c"VK_KHR_swapchain".as_ptr()];

// entry is stored because it's dynamic/loaded (!= from static/linked).
pub struct Renderer {
    instance: NonNull<Instance>,
    pub window: Window,
    pub surface: SurfaceKHR,
    pub device: RendererDevice,
    pub graphics_queue: Queue,
    pub present_queue: Queue,
}

impl Drop for Renderer {
    fn drop(&mut self) {
        unsafe {
            self.instance
                .as_ref()
                .surface_khr()
                .destroy_surface(self.surface, None);
            self.device.destroy_device(None);
        }
    }
}

impl Renderer {
    pub fn new(event_loop: &ActiveEventLoop, instance: &Instance) -> Renderer {
        // Create window
        let window = event_loop
            .create_window(
                Window::default_attributes()
                    .with_title("Vulkan project")
                    .with_inner_size(PhysicalSize::new(WIDTH, HEIGHT)),
            )
            .expect("Failed to create window");

        // Create surface
        let surface = unsafe {
            ash_window::create_surface(
                &instance.entry,
                &instance,
                window.display_handle().unwrap().into(),
                window.window_handle().unwrap().into(),
                None,
            )
            .expect("Failed to create surface.")
        };

        // Query physical device infos
        let infos = query_hardware(&instance, &surface);

        // Create device and queues
        let device = RendererDevice::new(instance, &surface, infos);
        let graphics_queue = unsafe { device.get_device_queue(device.graphics_idx, 0) };
        let present_queue = unsafe { device.get_device_queue(device.present_idx, 0) };

        // TODO : create swapchain

        Renderer {
            instance: NonNull::from(instance),
            window,
            surface,
            device,
            graphics_queue,
            present_queue,
        }
    }
}

// panics if no compatible hardware
fn query_hardware(instance: &Instance, surface: &SurfaceKHR) -> PhysicalDeviceInfos {
    // query all physical devices
    let physical_devices: Vec<PhysicalDeviceInfos> =
        unsafe { instance.enumerate_physical_devices() }
            .expect("Failed to query physical devices.")
            .into_iter()
            .filter_map(|physical_device| {
                query_physical_device(instance, surface, physical_device).ok()
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

pub struct PhysicalDeviceInfos {
    physical_device: PhysicalDevice,
    score: u32,
    capabilities: SurfaceCapabilitiesKHR,
    format: SurfaceFormatKHR,
    present_mode: PresentModeKHR,
    graphics_idx: u32,
    present_idx: u32,
}

// Query infos for a physical device (fails when the device is unsuitable)
fn query_physical_device(
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

    // swapchain extension
    let has_swapchain_extension = is_extension_available(EXTENSIONS[0], &available_extensions);
    if !has_swapchain_extension {
        return Err(());
    }

    // fetching swapchain capabilities, format, present_mode
    // unwrap()s shouldn't fail if device because device has swapchain extension
    let capabilities = unsafe {
        instance
            .surface_khr()
            .get_physical_device_surface_capabilities(physical_device, *surface)
    }
    .unwrap();
    let formats = unsafe {
        instance
            .surface_khr()
            .get_physical_device_surface_formats(physical_device, *surface)
    }
    .unwrap();
    let present_modes = unsafe {
        instance
            .surface_khr()
            .get_physical_device_surface_present_modes(physical_device, *surface)
    }
    .unwrap();
    if formats.is_empty() || present_modes.is_empty() {
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

    // SRGB_8 format is prefered (todo)
    let format = formats[0];

    // Choose FIFO mode (todo)
    let present_mode = present_modes[0];

    Ok(PhysicalDeviceInfos {
        physical_device,
        score,
        capabilities,
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
