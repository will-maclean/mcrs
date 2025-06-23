use crate::{texture, State};

use winit::application::ApplicationHandler;
use winit::event::{DeviceEvent, WindowEvent};
use winit::event_loop::ActiveEventLoop;
use winit::window::{Window, WindowId};

pub struct StateApplication {
    pub state: Option<State>,
}

impl Default for StateApplication {
    fn default() -> Self {
        Self::new()
    }
}

impl StateApplication {
    pub fn new() -> Self {
        Self { state: None }
    }
}

impl<T: 'static> ApplicationHandler<T> for StateApplication {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window = event_loop
            .create_window(Window::default_attributes().with_title("MCRS"))
            .unwrap();
        self.state = Some(State::new(window));
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        if let Some(state) = self.state.as_mut() {
            if state.window.id() == window_id {
                state.input(&event);
                match event {
                    WindowEvent::CloseRequested => {
                        event_loop.exit();
                        state.running = false;
                    }
                    WindowEvent::Resized(physical_size) => {
                        state.resize(physical_size);
                        state.depth_texture = texture::DepthTexture::new(
                            &state.device,
                            &state.config,
                            "depth_texture",
                        );
                    }
                    _ => {}
                }
            }
        }
    }

    fn device_event(
        &mut self,
        _: &ActiveEventLoop,
        _: winit::event::DeviceId,
        event: winit::event::DeviceEvent,
    ) {
        if let Some(state) = self.state.as_mut() {
            if let DeviceEvent::MouseMotion { delta: (dx, dy) } = event {
                if state.mouse_pressed {
                    state.camera_controller.process_mouse(dx, dy);
                }
            }
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        let window = self.state.as_ref().unwrap().window();
        window.request_redraw();
    }
}
