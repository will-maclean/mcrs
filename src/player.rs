use std::time::Duration;

use cgmath::{Point3, Rad, Vector3, Zero};

pub const GRAVITY: f32 = 9.8; // blocks / s^2

pub trait Entity {
    //TODO: will definitely take in other stuff as well
    fn update(&mut self, dt: Duration);

    //TODO: will definitely take in other stuff as well
    fn input(&mut self);
}

pub struct Player {
    pos: Point3<f32>,
    vel: Vector3<f32>,
    pitch: Rad<f32>,
    yaw: Rad<f32>,
    on_ground: bool,
}

impl Player {
    pub fn new(pos: Point3<f32>, pitch: Rad<f32>, yaw: Rad<f32>) -> Self {
        Self {
            pos,
            pitch,
            yaw,
            on_ground: true,
            vel: Vector3::zero(),
        }
    }

    fn detect_on_block(&self) -> bool {
        true
    }
}

impl Entity for Player {
    fn update(&mut self, dt: Duration) {
        self.on_ground = self.detect_on_block();

        if !self.on_ground {
            // update velocity
            self.vel.z -= dt.as_secs_f32() * GRAVITY;
        }
        // Note - if we are self.on_ground, then we
        // handle updates to the velocity from inputs
        // in the handle_input method
    }

    fn input(&mut self) {}
}
