mod renderer;
mod vk_loader;

use renderer::Renderer;
use vk_loader::Loader;
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::raw_window_handle::{HasDisplayHandle, RawDisplayHandle};
use winit::window::WindowId;

// Holds the application technical details
// Warning : Make sure the renderer drops before the loader => search "rust drop order"
struct App {
    renderer: Option<Renderer>,
    loader: Loader,
}

impl App {
    fn new(event_loop: &EventLoop<()>) -> App {
        let display_handle: RawDisplayHandle = event_loop
            .owned_display_handle()
            .display_handle()
            .expect("Failed to get display handle.")
            .into();
        App {
            loader: Loader::new(display_handle),
            renderer: None,
        }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        self.renderer = Some(Renderer::new(event_loop, &self.loader));
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
                self.renderer.as_ref().unwrap().window.request_redraw();
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
