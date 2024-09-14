mod device;

use std::ptr::NonNull;

use crate::instance::Instance;
use ash::vk::{PhysicalDevice, PhysicalDeviceType, Queue, QueueFlags, SurfaceKHR};
use device::RendererDevice;
use winit::{
    dpi::PhysicalSize,
    event_loop::ActiveEventLoop,
    raw_window_handle::{HasDisplayHandle, HasWindowHandle},
    window::Window,
};

const WIDTH: u32 = 800;
const HEIGHT: u32 = 800;

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

        // Select appropriate device for rendering on the surface
        let device = select_device(&instance, &surface);
        let graphics_queue = unsafe { device.get_device_queue(device.graphics_idx, 0) };
        let present_queue = unsafe { device.get_device_queue(device.present_idx, 0) };

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

fn select_device(instance: &Instance, surface: &SurfaceKHR) -> RendererDevice {
    // enumerate physical devices
    let physical_device_list = unsafe {
        instance
            .enumerate_physical_devices()
            .expect("Failed to query physical devices.")
    };

    // select one that suits our needs
    let chosen_one = physical_device_list
        .iter()
        .max_by_key(|physical_device| score_suitability(instance, physical_device, surface))
        .expect("No physical devices implement Vulkan.");
    if score_suitability(instance, chosen_one, surface) == 0 {
        panic!("No suitable device found.");
    }

    // construct device
    RendererDevice::new(instance, chosen_one, &surface)
}

// 0 means unsuitable
fn score_suitability(
    instance: &Instance,
    physical_device: &PhysicalDevice,
    surface: &SurfaceKHR,
) -> u32 {
    let mut score = 0;

    let queue_families =
        unsafe { instance.get_physical_device_queue_family_properties(*physical_device) };

    // graphic queue check : early return 0 if no queue family for graphics
    let can_do_graphics = queue_families
        .iter()
        .any(|queue_family| queue_family.queue_flags.contains(QueueFlags::GRAPHICS));
    if !can_do_graphics {
        return 0;
    }

    // surface support check : early return 0 if no queue family can present on this surface
    let can_present = queue_families.iter().enumerate().any(|(idx, _)| unsafe {
        instance
            .surface_khr()
            .get_physical_device_surface_support(*physical_device, idx as u32, *surface)
            .unwrap()
    });
    if !can_present {
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
