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
    #[allow(dead_code)]
    Entity,
    None,
}

pub fn block_contains(block_pos: Point3<i32>, test_pos: Point3<f32>) -> bool {
    let block_pos = block_pos.cast::<f32>().unwrap();

    test_pos.x >= block_pos.x
        && test_pos.x <= (block_pos.x + 1.0)
        && test_pos.y >= block_pos.y
        && test_pos.y <= (block_pos.y + 1.0)
        && test_pos.z >= block_pos.z
        && test_pos.z <= (block_pos.z + 1.0)
}

pub fn get_colliding_face(
    ray: Ray,
    collision_pos: Point3<f32>,
    block_loc: Point3<i32>,
) -> Option<BlockFace> {
    // Useful: https://gdbooks.gitbooks.io/3dcollisions/content/Chapter3/raycast_aabb.html

    // First things first, make sure the
    // block contains this point
    if !block_contains(block_loc, collision_pos) {
        log::warn!("get_colliding_face called with collision pos outside block. Ray={:?}, collision={:?}, block={:?}", ray, collision_pos, block_loc);
        return None;
    }

    // we can use the signs of the ray vec to immediately discard
    // three faces. If ray_vec.x is negative, we know there is
    // no way the ray has collided with the negative x face. See:
    //
    //  ^ y axis
    //  |
    //  |----------|
    //  |          |
    //  |          |    <----- ray (ray.x is negative)
    //  |          |
    //  |----------|-> x axis
    //
    //  x neg      x pos
    //  face        face
    //
    //  We can apply this logic with all 3 dims

    let block_loc = block_loc.cast::<f32>().unwrap();
    let t_min_x = (block_loc.x - ray.pos.x) / ray.dir.x;
    let t_max_x = (block_loc.x + 1.0 - ray.pos.x) / ray.dir.x;
    let t_min_y = (block_loc.y - ray.pos.y) - ray.dir.y;
    let t_max_y = (block_loc.y + 1.0 - ray.pos.y) / ray.dir.y;
    let t_min_z = (block_loc.z - ray.pos.z) - ray.dir.z;
    let t_max_z = (block_loc.z + 1.0 - ray.pos.z) / ray.dir.z;

    let (x_pos, x_dist) = if ray.dir.x.signum() > 0.0 {
        (true, t_min_x)
    } else {
        (false, t_max_x)
    };

    let (y_pos, y_dist) = if ray.dir.y.signum() > 0.0 {
        (true, t_min_y)
    } else {
        (false, t_max_y)
    };

    let (z_pos, z_dist) = if ray.dir.z.signum() > 0.0 {
        (true, t_min_z)
    } else {
        (false, t_max_z)
    };

    match argmax(&vec![x_dist, y_dist, z_dist]).unwrap() {
        0 => {
            if x_pos {
                Some(BlockFace::XNeg)
            } else {
                Some(BlockFace::XPos)
            }
        }
        1 => {
            if y_pos {
                Some(BlockFace::YNeg)
            } else {
                Some(BlockFace::YPos)
            }
        }
        2 => {
            if z_pos {
                Some(BlockFace::ZNeg)
            } else {
                Some(BlockFace::ZPos)
            }
        }
        _ => panic!("argmax should be 0, 1, or 2"),
    }
}

pub fn argmax<T: TotalOrder>(v: &[T]) -> Option<usize> {
    v.iter()
        .enumerate()
        .max_by(|(_, a), (_, b)| a.total_cmp(b))
        .map(|(index, _)| index)
}

pub fn argmin<T: TotalOrder>(v: &[T]) -> Option<usize> {
    v.iter()
        .enumerate()
        .min_by(|(_, a), (_, b)| a.total_cmp(b))
        .map(|(index, _)| index)
}

#[cfg(test)]
mod tests {
    use cgmath::Rad;

    use super::*;

    #[test]
    fn test_argmin() {
        assert_eq!(argmin(&vec![0.0, 1.0, 2.0]), Some(0));
        assert_eq!(argmin(&vec![0.0, -1.0, 2.0]), Some(1));
    }

    #[test]
    fn test_argmax() {
        assert_eq!(argmax(&vec![0.0, 1.0, 2.0]), Some(2));
        assert_eq!(argmax(&vec![0.0, -1.0, 2.0]), Some(2));
    }

    #[test]
    fn test_get_colliding_face() {
        let ray = Ray::from(&Camera::new(Point3::new(0.0, 0.0, 0.0), Rad(0.0), Rad(0.0)));

        assert_eq!(
            get_colliding_face(
                ray.clone(),
                Point3::new(1.01, 0.5, 0.5),
                Point3::new(1, 0, 0)
            ),
            Some(BlockFace::XNeg),
        );
        assert_eq!(
            get_colliding_face(ray, Point3::new(1.99, 0.5, 0.5), Point3::new(1, 0, 0)),
            Some(BlockFace::XNeg),
        );
    }

    #[test]
    fn test_block_contains() {
        let cases = vec![
            // block pos, test pos, result
            (Point3::new(0, 0, 0), Point3::new(0.1, 0.1, 0.1), true),
            (Point3::new(0, 0, 0), Point3::new(1.1, 0.1, 0.1), false),
            (Point3::new(0, 0, 0), Point3::new(0.1, 1.1, 0.1), false),
            (Point3::new(0, 0, 0), Point3::new(0.1, 0.1, 1.1), false),
            (Point3::new(0, 0, 0), Point3::new(-1.1, 0.1, 0.1), false),
            (Point3::new(0, 0, 0), Point3::new(0.1, -1.1, 0.1), false),
            (Point3::new(0, 0, 0), Point3::new(0.1, 0.1, -1.1), false),
        ];

        for (block, test, result) in cases {
            assert_eq!(block_contains(block, test), result);
        }
    }
}
