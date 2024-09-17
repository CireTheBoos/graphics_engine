mod device;
mod swapchain;

use crate::instance::Instance;
use ash::vk::{ImageView, Queue, SurfaceKHR};
use device::RendererDevice;
use swapchain::RendererSwapchain;

use std::ptr::NonNull;
use winit::{
    dpi::PhysicalSize,
    event_loop::ActiveEventLoop,
    raw_window_handle::{HasDisplayHandle, HasWindowHandle},
    window::Window,
};

const WIDTH: u32 = 800;
const HEIGHT: u32 = 800;

// Render images on screen from model data
pub struct Renderer {
    instance: NonNull<Instance>,
    pub window: Window,
    pub surface: SurfaceKHR,
    pub device: RendererDevice,
    pub swapchain: RendererSwapchain,
    pub image_views: Vec<ImageView>,
    pub graphics_queue: Queue,
    pub present_queue: Queue,
}

// Destroy views, swapchain, surface (order matters)
impl Drop for Renderer {
    fn drop(&mut self) {
        unsafe {
            for image_view in &self.image_views {
                self.device.destroy_image_view(*image_view, None);
            }
            self.device
                .swapchain_khr()
                .destroy_swapchain(*self.swapchain, None);
            self.instance
                .as_ref()
                .surface_khr()
                .destroy_surface(self.surface, None);
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
                instance.entry(),
                &instance,
                window.display_handle().unwrap().into(),
                window.window_handle().unwrap().into(),
                None,
            )
            .expect("Failed to create surface.")
        };

        // Create device and queues
        let (device, infos) = RendererDevice::new(instance, &surface);
        let graphics_queue = unsafe { device.get_device_queue(device.graphics_idx, 0) };
        let present_queue = unsafe { device.get_device_queue(device.present_idx, 0) };

        // Create swapchain
        let swapchain = RendererSwapchain::new(&device, &surface, infos);

        let image_views = swapchain.get_image_views(&device);

        Renderer {
            instance: NonNull::from(instance),
            window,
            surface,
            device,
            swapchain,
            image_views,
            graphics_queue,
            present_queue,
        }
    }
}
