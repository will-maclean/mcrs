use std::default;

use cgmath::{Point3, Vector3};

use crate::camera::Camera;

#[derive(Debug, Clone)]
pub struct Ray {
    pub pos: Point3<f32>,
    pub dir: Vector3<f32>,
    pub max_dist: f32,
    pub n_tests: usize,
}

impl From<&Camera> for Ray {
    fn from(value: &Camera) -> Self {
        Self {
            pos: value.position,
            dir: value.front(),
            max_dist: 5.0,
            n_tests: 100,
        }
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
    Block {
        loc: Point3<i32>,
        face: BlockFace,
        dist: f32,
    },
    Entity,
    None,
}
