use std::time::{Duration, Instant};

use winit::{event_loop::EventLoop, platform::pump_events::EventLoopExtPumpEvents};

use crate::app;

pub struct MCRS<T: 'static> {
    state_app: app::StateApplication,
    last_render_time: instant::Instant,
    last_update_time: instant::Instant,
    event_loop: EventLoop<T>,
}

impl<T> MCRS<T> {
    const DESIRED_FPS: u32 = 60;

    pub fn new(window_state: app::StateApplication, event_loop: EventLoop<T>) -> Self {
        Self {
            state_app: window_state,
            event_loop,
            last_render_time: Instant::now(),
            last_update_time: Instant::now(),
        }
    }

    pub fn run(&mut self) {
        //NOTE: if we start cooking CPUs, can limit the update rate
        // as well.

        let update_timestep = Duration::from_secs_f64(1.0 / Self::DESIRED_FPS as f64);
        let mut prev_time = instant::Instant::now();
        let mut accum_time = Duration::ZERO;

        loop {
            let curr_time = instant::Instant::now();
            let elapsed_time = curr_time - prev_time;
            prev_time = curr_time;
            accum_time += elapsed_time;

            // Let's arbitrarily say 1/3 of the desired tick
            // time is devoted to processing inputs. No idea if
            // this is adequately smart or completely stupid.
            self.input(update_timestep / 3);

            while accum_time >= update_timestep {
                let _ = self.update();

                accum_time -= update_timestep;
            }

            let _ = self.render();
        }
    }

    fn input(&mut self, duration: Duration) {
        let start = Instant::now();

        while (Instant::now() - start) > duration {
            let _ = self
                .event_loop
                .pump_app_events(Some(Duration::ZERO), &mut self.state_app);
        }
    }

    fn update(&mut self) -> Result<(), ()> {
        Ok(())
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
}
