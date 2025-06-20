use cgmath::{Point3, Vector3};

use crate::camera::Camera;

#[derive(Debug, Clone)]
pub struct Ray {
    pub pos: Point3<f32>,
    pub dir: Vector3<f32>,
}

impl From<&Camera> for Ray {
    fn from(value: &Camera) -> Self {
        todo!()
    }
}

#[derive(Debug, Clone, Copy)]
pub enum BlockFace {
    XPos,
    XNeg,
    YPos,
    YNeg,
    ZPos,
    ZNeg,
}

#[derive(Debug, Clone)]
pub enum RayResult {
    Block { loc: Point3<i32>, face: BlockFace },
    Entity,
    None,
}
