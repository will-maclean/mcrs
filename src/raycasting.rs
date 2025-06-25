use cgmath::{num_traits::float::TotalOrder, Point3, Vector3};

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

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BlockFace {
    XPos,
    XNeg,
    YPos,
    YNeg,
    ZPos,
    ZNeg,
}

impl BlockFace {
    pub fn adjacent_loc_from(&self, loc: Point3<i32>) -> Point3<i32> {
        let mut new_loc = loc.clone();
        match self {
            BlockFace::XPos => new_loc.x += 1,
            BlockFace::XNeg => new_loc.x -= 1,
            BlockFace::YPos => new_loc.y += 1,
            BlockFace::YNeg => new_loc.y -= 1,
            BlockFace::ZPos => new_loc.z += 1,
            BlockFace::ZNeg => new_loc.z -= 1,
        }

        new_loc
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum RayResult {
    Block {
        loc: Point3<i32>,
        face: BlockFace,
        dist: f32,
    },
    Entity,
    None,
}

pub fn get_colliding_face(
    _camera_pos: Point3<f32>,
    collision_pos: Point3<f32>,
    block_loc: Point3<i32>,
) -> Option<BlockFace> {
    // For now, just return the face clostest to the intersecting
    // point.
    // FIXME: this is a gross disgusting method that will
    // cause weird behaviour - might work well enough for now though
    // TODO: need to check that collision_pos is contained in the block
    let block_loc = block_loc.cast::<f32>().unwrap();
    let x_pos_dist = block_loc.x + 1.0 - collision_pos.x;
    let x_neg_dist = collision_pos.x - block_loc.x;
    let y_pos_dist = block_loc.y + 1.0 - collision_pos.y;
    let y_neg_dist = collision_pos.y - block_loc.y;
    let z_pos_dist = block_loc.z + 1.0 - collision_pos.z;
    let z_neg_dist = collision_pos.z - block_loc.z;

    let dists = vec![
        x_pos_dist, x_neg_dist, y_pos_dist, y_neg_dist, z_pos_dist, z_neg_dist,
    ];

    if let Some(amin) = argmin(&dists) {
        match amin {
            0 => Some(BlockFace::XPos),
            1 => Some(BlockFace::XNeg),
            2 => Some(BlockFace::YPos),
            3 => Some(BlockFace::YNeg),
            4 => Some(BlockFace::ZPos),
            5 => Some(BlockFace::ZNeg),
            _ => None,
        }
    } else {
        None
    }
}

pub fn argmin<T: TotalOrder>(v: &[T]) -> Option<usize> {
    v.iter()
        .enumerate()
        .min_by(|(_, a), (_, b)| a.total_cmp(b))
        .map(|(index, _)| index)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_argmin() {
        assert_eq!(argmin(&vec![0.0, 1.0, 2.0]), Some(0));
        assert_eq!(argmin(&vec![0.0, -1.0, 2.0]), Some(1));
    }

    #[test]
    fn test_get_colliding_face() {
        assert_eq!(
            get_colliding_face(
                Point3::new(0.0, 0.0, 0.0),
                Point3::new(1.01, 0.5, 0.5),
                Point3::new(1, 0, 0)
            ),
            Some(BlockFace::XNeg),
        );

        assert_eq!(
            get_colliding_face(
                Point3::new(0.0, 0.0, 0.0),
                Point3::new(1.99, 0.5, 0.5),
                Point3::new(1, 0, 0)
            ),
            Some(BlockFace::XPos),
        );

        assert_eq!(
            get_colliding_face(
                Point3::new(0.0, 0.0, 0.0),
                Point3::new(1.5, 0.1, 0.5),
                Point3::new(1, 0, 0)
            ),
            Some(BlockFace::YNeg),
        );

        assert_eq!(
            get_colliding_face(
                Point3::new(0.0, 0.0, 0.0),
                Point3::new(1.5, 0.9, 0.5),
                Point3::new(1, 0, 0)
            ),
            Some(BlockFace::YPos),
        );

        assert_eq!(
            get_colliding_face(
                Point3::new(0.0, 0.0, 0.0),
                Point3::new(1.5, 0.5, 0.1),
                Point3::new(1, 0, 0)
            ),
            Some(BlockFace::ZNeg),
        );

        assert_eq!(
            get_colliding_face(
                Point3::new(0.0, 0.0, 0.0),
                Point3::new(1.5, 0.5, 0.9),
                Point3::new(1, 0, 0)
            ),
            Some(BlockFace::ZPos),
        );
    }
}
