mod context;

// Vkc = Vulkan custom -> Not provided by ash or Vulkan but implemented by me
use context::Context;

use winit::application::ApplicationHandler;
use winit::dpi::PhysicalSize;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::raw_window_handle::{HasDisplayHandle, RawDisplayHandle};
use winit::window::{Window, WindowId};

const WIDTH: u32 = 800;
const HEIGHT: u32 = 800;

// Hold the application technical details
struct App {
    window: Option<Window>,
    ctx: Context,
}

impl App {
    fn new(event_loop: &EventLoop<()>) -> App {
        let display_handle: RawDisplayHandle = event_loop
            .owned_display_handle()
            .display_handle()
            .expect("Failed to get display handle.")
            .into();
        App {
            window: None,
            ctx: Context::new(display_handle),
        }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        self.window = Some(
            event_loop
                .create_window(
                    Window::default_attributes()
                        .with_title("Vulkan project")
                        .with_inner_size(PhysicalSize::new(WIDTH, HEIGHT)),
                )
                .expect("Failed to create window"),
        );
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => {
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

fn main() {
    // STEP 1. : Setup the event_loop to receive events from the OS and window
    let event_loop = EventLoop::new().expect("Failed to create event_loop");
    // ControlFlow::Poll continuously runs the event loop, even if the OS hasn't
    // dispatched any events. This is ideal for games and similar applications.
    event_loop.set_control_flow(ControlFlow::Poll);

    // STEP 2. : Create App and run it with the event_loop
    let mut app = App::new(&event_loop);
    event_loop.run_app(&mut app).expect("Failed to run the app");
}
