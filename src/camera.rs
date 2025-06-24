use cgmath::*;
use std::{f32::consts::FRAC_PI_2, time::Duration};
use winit::{event::*, keyboard::KeyCode};

#[rustfmt::skip]
pub const OPENGL_TO_WGPU_MATRIX: cgmath::Matrix4<f32> = cgmath::Matrix4::from_cols(
    cgmath::Vector4::new(1.0, 0.0, 0.0, 0.0),
    cgmath::Vector4::new(0.0, 1.0, 0.0, 0.0),
    cgmath::Vector4::new(0.0, 0.0, 0.5, 0.0),
    cgmath::Vector4::new(0.0, 0.0, 0.5, 1.0),
);

#[rustfmt::skip]
pub const WGPU_TO_WORLD_MATRIX: cgmath::Matrix4<f32> = cgmath::Matrix4::from_cols(
    cgmath::Vector4::new(-1.0, 0.0, 0.0, 0.0),
    cgmath::Vector4::new(0.0, 1.0, 0.0, 0.0),
    cgmath::Vector4::new(0.0, 0.0, 1.0, 0.0),
    cgmath::Vector4::new(0.0, 0.0, 0.0, 1.0),
);

const SAFE_FRAC_PI_2: f32 = FRAC_PI_2 - 0.0001;

pub struct Camera {
    pub position: Point3<f32>,
    pub yaw: Rad<f32>,
    pub pitch: Rad<f32>,
}

impl Camera {
    pub fn new<V: Into<Point3<f32>>, Y: Into<Rad<f32>>, P: Into<Rad<f32>>>(
        position: V,
        yaw: Y,
        pitch: P,
    ) -> Self {
        Self {
            position: position.into(),
            yaw: yaw.into(),
            pitch: pitch.into(),
        }
    }

    pub fn calc_matrix(&self) -> Matrix4<f32> {
        Matrix4::look_to_rh(self.position, self.front(), Vector3::unit_z())
    }

    pub fn print_info(&self) {
        println!(
            "Camera Position: {:?}, Yaw: {}, Pitch: {}",
            self.position, self.yaw.0, self.pitch.0
        );
    }

    pub fn front(&self) -> Vector3<f32> {
        let (sin_pitch, cos_pitch) = self.pitch.sin_cos();
        let (sin_yaw, cos_yaw) = self.yaw.sin_cos();

        Vector3::new(cos_pitch * cos_yaw, cos_pitch * sin_yaw, sin_pitch).normalize()
    }
}

pub struct Projection {
    aspect: f32,
    pub fovy: Rad<f32>,
    znear: f32,
    zfar: f32,
}

impl Projection {
    pub fn new<F: Into<Rad<f32>>>(width: u32, height: u32, fovy: F, znear: f32, zfar: f32) -> Self {
        Self {
            aspect: width as f32 / height as f32,
            fovy: fovy.into(),
            znear,
            zfar,
        }
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.aspect = width as f32 / height as f32;
    }

    pub fn calc_matrix(&self) -> Matrix4<f32> {
        OPENGL_TO_WGPU_MATRIX * perspective(self.fovy, self.aspect, self.znear, self.zfar)
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraUniform {
    pub view_proj: [[f32; 4]; 4],
}

impl Default for CameraUniform {
    fn default() -> Self {
        Self::new()
    }
}

impl CameraUniform {
    pub fn new() -> Self {
        use cgmath::SquareMatrix;

        Self {
            view_proj: cgmath::Matrix4::identity().into(),
        }
    }

    pub fn update_view_proj(&mut self, camera: &Camera, projection: &Projection) {
        self.view_proj =
            (projection.calc_matrix() * WGPU_TO_WORLD_MATRIX * camera.calc_matrix()).into();
    }
}

pub struct CameraController {
    amount_left: f32,
    amount_right: f32,
    amount_up: f32,
    amount_down: f32,
    amount_forward: f32,
    amount_backward: f32,
    rotate_horizontal: f32,
    rotate_vertical: f32,
    sensitivity: f32,
    speed: f32,
    pub forward: Vector3<f32>,
    pub right: Vector3<f32>,
}

impl CameraController {
    pub fn new(speed: f32, sensitivity: f32) -> Self {
        Self {
            speed,
            sensitivity,
            amount_left: 0.0,
            amount_right: 0.0,
            amount_up: 0.0,
            amount_down: 0.0,
            amount_forward: 0.0,
            amount_backward: 0.0,
            rotate_horizontal: 0.0,
            rotate_vertical: 0.0,
            forward: Vector3::zero(),
            right: Vector3::zero(),
        }
    }

    pub fn process_keyboard(&mut self, key: KeyCode, state: ElementState) {
        let amount = if state == ElementState::Pressed {
            1.0
        } else {
            0.0
        };

        match key {
            KeyCode::KeyW | KeyCode::ArrowUp => {
                self.amount_forward = amount;
            }
            KeyCode::KeyS | KeyCode::ArrowDown => {
                self.amount_backward = amount;
            }
            KeyCode::KeyA | KeyCode::ArrowLeft => {
                self.amount_left = amount;
            }
            KeyCode::KeyD | KeyCode::ArrowRight => {
                self.amount_right = amount;
            }
            KeyCode::Space => {
                self.amount_up = amount;
            }
            KeyCode::ShiftLeft => {
                self.amount_down = amount;
            }
            _ => (),
        }
    }

    pub fn process_mouse(&mut self, mouse_dx: f64, mouse_dy: f64) {
        self.rotate_horizontal = mouse_dx as f32;
        self.rotate_vertical = mouse_dy as f32;
    }

    pub fn process_scroll(&mut self, _delta: &MouseScrollDelta) {}

    pub fn update_camera(&mut self, camera: &mut Camera, dt: Duration) {
        let dt = dt.as_secs_f32();

        // Move forward/backward and left/right
        let (yaw_sin, yaw_cos) = camera.yaw.sin_cos();
        self.forward = Vector3::new(yaw_cos, yaw_sin, 0.0).normalize();
        self.right = Vector3::new(-yaw_sin, yaw_cos, 0.0).normalize();
        camera.position +=
            self.forward * (self.amount_forward - self.amount_backward) * self.speed * dt;
        camera.position += self.right * (self.amount_right - self.amount_left) * self.speed * dt;
        camera.position.z += (self.amount_up - self.amount_down) * self.speed * dt;

        // Rotate
        camera.yaw += Rad(self.rotate_horizontal) * self.sensitivity * dt;
        camera.pitch += Rad(self.rotate_vertical) * self.sensitivity * dt;

        // If process_mouse isn't called every frame, these values
        // will not get set to zero, and the camera will rotate
        // when moving in a non cardinal direction.
        self.rotate_horizontal = 0.0;
        self.rotate_vertical = 0.0;

        // Keep the camera's angle from going too high/low.
        if camera.pitch < -Rad(SAFE_FRAC_PI_2) {
            camera.pitch = -Rad(SAFE_FRAC_PI_2);
        } else if camera.pitch > Rad(SAFE_FRAC_PI_2) {
            camera.pitch = Rad(SAFE_FRAC_PI_2);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_forward() {
        let camera = Camera::new(Point3::new(0.0, 0.0, 0.0), Rad(0.0), Rad(0.0));

        assert_eq!(camera.front(), Vector3::new(1.0, 0.0, 0.0));
    }
}
