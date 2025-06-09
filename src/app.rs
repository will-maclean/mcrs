use crate::texture;
use crate::State;

use wgpu::{Adapter, Device, Instance, PresentMode, Queue, Surface, SurfaceCapabilities};
use winit::application::ApplicationHandler;
use winit::event::{DeviceEvent, WindowEvent};
use winit::event_loop::ActiveEventLoop;
use winit::event_loop::EventLoop;
use winit::window::{Window, WindowId};

pub struct StateApplication {
    state: Option<State>,
}

impl StateApplication {
    pub fn new() -> Self {
        Self { state: None }
    }
}

impl ApplicationHandler for StateApplication {
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
                //TODO: is there anything to do with the return code??
                let _ = state.input(&event);
                match event {
                    WindowEvent::CloseRequested => {
                        event_loop.exit();
                    }
                    WindowEvent::Resized(physical_size) => {
                        state.resize(physical_size);
                        state.depth_texture = texture::Texture::create_depth_texture(
                            &state.device,
                            &state.config,
                            "depth_texture",
                        );
                    }
                    WindowEvent::RedrawRequested => {
                        // state.window.request_redraw();

                        let now = instant::Instant::now();
                        let dt = now - state.last_render_time;
                        state.last_render_time = now;
                        state.update(dt);
                        state.debug_view.update_text(
                            format!(
                                "Debug View\nCamera pos: ({:.2}, {:.2}, {:.2})\nPitch: {:?}, Yaw: {:?}",
                                state.camera.position.x,
                                state.camera.position.y,
                                state.camera.position.z,
                                state.camera.pitch,
                                state.camera.yaw,
                            )
                            .as_str(),
                        );

                        match state.render() {
                            Ok(_) => {}
                            Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
                                // TODO: probably want to re-add size to the config
                                // state.resize(state.config.size)
                            }
                            // The system is out of memory, we should probably quit
                            Err(wgpu::SurfaceError::OutOfMemory | wgpu::SurfaceError::Other) => {
                                log::error!("OutOfMemory");
                                event_loop.exit();
                            }

                            // This happens when the a frame takes too long to present
                            Err(wgpu::SurfaceError::Timeout) => {
                                log::warn!("Surface timeout")
                            }
                        }
                    }
                    WindowEvent::MouseInput {
                        device_id: _,
                        state: btn_state,
                        button,
                    } => state.handle_mouse_button(button, btn_state.is_pressed()),
                    WindowEvent::MouseWheel { delta, .. } => state.handle_mouse_scroll(&delta),
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
            match event {
                DeviceEvent::MouseMotion { delta: (dx, dy) } => {
                    if state.mouse_pressed {
                        state.camera_controller.process_mouse(dx, dy);
                    }
                }
                _ => (),
            }
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        let window = self.state.as_ref().unwrap().window();
        window.request_redraw();
    }
}
