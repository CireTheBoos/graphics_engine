mod graphics_engine;
mod instance;
mod model;

use ash::vk::SurfaceKHR;
use graphics_engine::GraphicsEngine;
use instance::Instance;
use model::Model;

use winit::{
    application::ApplicationHandler,
    dpi::PhysicalSize,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    raw_window_handle::{HasDisplayHandle, HasWindowHandle, RawDisplayHandle},
    window::{Window, WindowId},
};

const TITLE: &str = "Renderer";
const WIDTH: u32 = 600;
const HEIGHT: u32 = 600;

// Interface with the OS => manage windows and events. Methods :
// - new(...) : Instantiate a vk instance capable of rendering to the display and a new model.
// - resumed(...) : Instantiate a window and a renderer to its inner surface.
// - window_event(event, ...) : Handle events.
struct App {
    instance: Instance,
    model: Model,
    window: Option<Window>,
    renderer: Option<GraphicsEngine>,
}

impl App {
    // Only instantiate vulkan, rendering is setup in "resumed()"
    fn new(event_loop: &EventLoop<()>) -> App {
        let raw_display_handle: RawDisplayHandle = event_loop
            .owned_display_handle()
            .display_handle()
            .expect("Failed to get display handle.")
            .into();
        App {
            instance: Instance::new(raw_display_handle),
            model: Model::new(),
            window: None,
            renderer: None,
        }
    }
}

// Trait to be able to receive events from event_loop
impl ApplicationHandler for App {
    // Create a window and its renderer
    // Called when a window become visible
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window = create_window(event_loop);
        let surface = create_surface(&self.instance, &window);
        let renderer = GraphicsEngine::new(&self.instance, surface);
        self.window = Some(window);
        self.renderer = Some(renderer);
    }

    // Only handles "Redraw" and "Close" requests
    fn window_event(&mut self, event_loop: &ActiveEventLoop, _: WindowId, event: WindowEvent) {
        if event_loop.exiting() {
            return;
        }
        match event {
            WindowEvent::CloseRequested => {
                self.renderer.as_mut().unwrap().destroy(&self.instance);
                self.renderer = None;
                self.window = None;
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => {
                self.model.step_if_enough_time();
                self.renderer.as_mut().unwrap().frame(&self.model.vertices);
                // Request "Redraw" again, making it loop as fast as possible
                self.window.as_ref().unwrap().request_redraw();
            }
            _ => {}
        }
    }
}

// Create basic window with TITLE, WIDTH, HEIGHT
fn create_window(event_loop: &ActiveEventLoop) -> Window {
    let window_attributes = Window::default_attributes()
        .with_title(TITLE)
        .with_inner_size(PhysicalSize::new(WIDTH, HEIGHT));
    event_loop
        .create_window(window_attributes)
        .expect("Failed to create window.")
}

// Get inner window as a surfaceKHR
fn create_surface(instance: &Instance, window: &Window) -> SurfaceKHR {
    unsafe {
        ash_window::create_surface(
            instance.entry(),
            instance,
            window.display_handle().unwrap().into(),
            window.window_handle().unwrap().into(),
            None,
        )
        .expect("Failed to create surface.")
    }
}

fn main() {
    // Create event_loop and app
    let event_loop = EventLoop::new().expect("Failed to create event loop.");
    event_loop.set_control_flow(ControlFlow::Poll);
    let mut app = App::new(&event_loop);

    // Run app on event_loop
    event_loop
        .run_app(&mut app)
        .expect("Failed to run the app.");
}
