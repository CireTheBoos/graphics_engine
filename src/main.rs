mod app;

use app::App;
use winit::{
    event_loop::{ControlFlow, EventLoop},
    raw_window_handle::{HasDisplayHandle, RawDisplayHandle},
};

fn main() {
    // Create event_loop and app
    let event_loop = EventLoop::new().expect("Failed to create event loop.");
    event_loop.set_control_flow(ControlFlow::Poll);
    let raw_display_handle: RawDisplayHandle = get_rdh_from_event_loop(&event_loop);
    let mut app = App::new(raw_display_handle);

    // Run app on event_loop
    event_loop
        .run_app(&mut app)
        .expect("Failed to run the app.");
}

fn get_rdh_from_event_loop(event_loop: &EventLoop<()>) -> RawDisplayHandle {
    event_loop
        .owned_display_handle()
        .display_handle()
        .expect("Failed to get display handle.")
        .into()
}
