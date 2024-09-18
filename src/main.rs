mod instance;
mod renderer;
mod shaders;

use ash::vk::SurfaceKHR;
use instance::Instance;
use renderer::Renderer;

use std::{marker::PhantomPinned, pin::pin};
use winit::{
    application::ApplicationHandler,
    dpi::PhysicalSize,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    raw_window_handle::{HasDisplayHandle, HasWindowHandle, RawDisplayHandle},
    window::{Window, WindowId},
};

const WIDTH: u32 = 600;
const HEIGHT: u32 = 600;

// Setup loop and handle events
struct App {
    renderer: Option<Renderer>,
    window: Option<Window>,
    instance: Instance,
    _pin: PhantomPinned,
}

impl App {
    fn new(event_loop: &EventLoop<()>) -> App {
        let raw_display_handle: RawDisplayHandle = event_loop
            .owned_display_handle()
            .display_handle()
            .expect("Failed to get display handle.")
            .into();
        App {
            renderer: None,
            window: None,
            instance: Instance::new(raw_display_handle),
            _pin: PhantomPinned,
        }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window = create_window(event_loop);
        let surface = create_surface(&self.instance, &window);
        let renderer = Renderer::new(&self.instance, surface);

        self.window = Some(window);
        self.renderer = Some(renderer);
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => {
                self.renderer.as_ref().unwrap().destroy(&self.instance);
                self.renderer = None;
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => {
                // Redraw the application.
                //
                // It's preferable for applications that do not render continuously to render in
                // this event rather than in AboutToWait, since rendering in here allows
                // the program to gracefully handle redraws requested by the OS.

                // Draw.

                // Queue a RedrawRequested event.
                //
                // You only need to call this if you've determined that you need to redraw in
                // applications which do not always need to. Applications that redraw continuously
                // can render here instead.
                self.window.as_ref().unwrap().request_redraw();
            }
            _event => {
                // println!("{:?}",_event);
            }
        }
    }
}

fn create_window(event_loop: &ActiveEventLoop) -> Window {
    // SPECIFY : title, inner size
    let attributes = Window::default_attributes()
        .with_title("Vulkan project")
        .with_inner_size(PhysicalSize::new(WIDTH, HEIGHT));
    // CREATE
    event_loop
        .create_window(attributes)
        .expect("Failed to create window.")
}

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
    // STEP 1. : Setup the event_loop to receive events from the OS and window
    let event_loop = EventLoop::new().expect("Failed to create event loop.");
    event_loop.set_control_flow(ControlFlow::Poll);

    // STEP 2. : Create App and run it with the event_loop
    // safe bc run_app() don't move "app" with the &mut given, so pinning hold
    let app = App::new(&event_loop);
    let app = pin!(app);
    event_loop
        .run_app(unsafe { app.get_unchecked_mut() })
        .expect("Failed to run the app.");
}
