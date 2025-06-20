use cgmath::{Point3, Vector3};

#[derive(Debug, Clone)]
pub struct Ray {
    pub pos: Point3<f32>,
    pub dir: Vector3<f32>,
}

#[derive(Debug, Clone)]
pub enum RayResult {
    Block,
    Entity,
    None,
}
