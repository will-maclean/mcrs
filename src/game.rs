use std::time::{Duration, Instant};

use winit::{event_loop::EventLoop, platform::pump_events::EventLoopExtPumpEvents};

use crate::app;

pub struct MCRS<T: 'static> {
    state_app: app::StateApplication,
    last_render_time: instant::Instant,
    last_update_time: instant::Instant,
    event_loop: EventLoop<T>,
    running: bool,
}

impl<T> MCRS<T> {
    const DESIRED_FPS: u32 = 60;

    pub fn new(window_state: app::StateApplication, event_loop: EventLoop<T>) -> Self {
        Self {
            state_app: window_state,
            event_loop,
            last_render_time: Instant::now(),
            last_update_time: Instant::now(),
            running: true,
        }
    }

    pub fn run(&mut self) {
        //NOTE: if we start cooking CPUs, can limit the update rate
        // as well.

        let update_timestep = Duration::from_secs_f64(1.0 / Self::DESIRED_FPS as f64);
        let mut prev_time = instant::Instant::now();
        let mut accum_time = Duration::ZERO;

        while self.running {
            let curr_time = instant::Instant::now();
            let elapsed_time = curr_time - prev_time;
            prev_time = curr_time;
            accum_time += elapsed_time;

            self.running &= self.input();

            while accum_time >= update_timestep {
                self.running &= self.update();

                accum_time -= update_timestep;
            }

            let _ = self.render();
        }

        self.close();
    }

    fn input(&mut self) -> bool {
        self.event_loop.pump_app_events(None, &mut self.state_app);

        if let Some(state) = self.state_app.state.as_ref() {
            return state.running;
        }

        false
    }

    fn update(&mut self) -> bool {
        if let Some(state) = self.state_app.state.as_mut() {
            let now = instant::Instant::now();
            let dt = now - self.last_update_time;
            self.last_update_time = now;

            state.update(dt);

            return state.running;
        }
        false
    }

    fn render(&mut self) -> Result<(), ()> {
        if let Some(state) = self.state_app.state.as_mut() {
            self.last_render_time = instant::Instant::now();
            state.debug_view.update_text(
            format!(
                "Debug View\nCamera pos: ({:.2}, {:.2}, {:.2})\nPitch: {:?}, Yaw: {:?}\nCamera forward: ({:.2}, {:.2}, {:.2})\nCamera right: ({:.2}, {:.2}, {:.2})",
                state.camera.position.x,
                state.camera.position.y,
                state.camera.position.z,
                state.camera.pitch,
                state.camera.yaw,
                state.camera_controller.forward.x,
                state.camera_controller.forward.y,
                state.camera_controller.forward.z,
                state.camera_controller.right.x,
                state.camera_controller.right.y,
                state.camera_controller.right.z,
            )
            .as_str(),
        );

            match state.render() {
                Ok(_) => Ok(()),
                Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
                    // TODO: probably want to re-add size to the config
                    // state.resize(state.config.size)
                    Ok(())
                }
                // The system is out of memory, we should probably quit
                Err(wgpu::SurfaceError::OutOfMemory | wgpu::SurfaceError::Other) => {
                    log::error!("OutOfMemory");
                    Err(())
                }

                // This happens when the a frame takes too long to present
                Err(wgpu::SurfaceError::Timeout) => {
                    log::warn!("Surface timeout");
                    Ok(())
                }
            }
        } else {
            Err(())
        }
    }

    fn close(&mut self) {}
}
