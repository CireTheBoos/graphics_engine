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
    event_loop::ActiveEventLoop,
    raw_window_handle::{HasDisplayHandle, HasWindowHandle, RawDisplayHandle},
    window::{Window, WindowId},
};

const TITLE: &str = "Renderer";
const WIDTH: u32 = 600;
const HEIGHT: u32 = 600;

pub struct App {
    instance: Instance,
    model: Model,
    // Rendering
    window: Option<Window>,
    graphics_engine: Option<GraphicsEngine>,
}

impl App {
    pub fn new(raw_display_handle: RawDisplayHandle) -> App {
        App {
            instance: Instance::new(raw_display_handle),
            model: Model::new(),
            window: None,
            graphics_engine: None,
        }
    }

    pub fn setup_rendering(&mut self, event_loop: &ActiveEventLoop) {
        let window = create_window(event_loop);
        let surface = create_surface(&self.instance, &window);
        let graphics_engine = GraphicsEngine::new(&self.instance, surface);
        self.window = Some(window);
        self.graphics_engine = Some(graphics_engine);
    }

    pub fn close(&mut self) {
        self.graphics_engine
            .as_mut()
            .unwrap()
            .destroy(&self.instance);
        self.graphics_engine = None;
        self.window = None;
    }

    pub fn redraw(&mut self) {
        self.model.step_if_enough_time();
        self.graphics_engine
            .as_mut()
            .unwrap()
            .frame(&self.model.squares, &self.model.camera);
        // Request "Redraw" again, making it loop as fast as possible
        self.window.as_ref().unwrap().request_redraw();
    }
}

// Trait to be able to receive events from event_loop
impl ApplicationHandler for App {
    // Called when we resume the app
    // Rendering should be setup here according to winit doc
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        self.setup_rendering(event_loop);
    }

    // Handles "Redraw" and "Close" requests
    fn window_event(&mut self, event_loop: &ActiveEventLoop, _: WindowId, event: WindowEvent) {
        if event_loop.exiting() {
            return;
        }
        match event {
            WindowEvent::CloseRequested => {
                self.close();
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => {
                self.redraw();
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
