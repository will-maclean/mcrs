use std::collections::{HashMap, HashSet};

use crate::{camera, model};
use cgmath::{prelude::*, Point2, Vector2};
use log::debug;

const CHUNK_WIDTH: usize = 16;
const CHUNK_HEIGHT: usize = 256;
const BOTTOM_DEPTH: i32 = -12;

#[derive(Debug, Clone, Copy)]
pub enum BlockType {
    Dirt,
    Weird,
}

impl BlockType {
    fn tex_label(&self) -> &'static str {
        match self {
            Self::Dirt => "stone",
            Self::Weird => "weird",
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct Block {
    origin_x: i32,
    origin_y: i32,
    origin_z: i32,
    block_type: BlockType,
}

#[derive(Debug, Clone)]
pub struct Chunk {
    origin: Point2<i32>,
    blocks: [[[Option<Block>; CHUNK_WIDTH]; CHUNK_WIDTH]; CHUNK_HEIGHT],
}

impl Chunk {
    pub fn gen_instances(&self) -> Vec<model::RenderInstance> {
        self.blocks
            .iter()
            .flatten()
            .flatten()
            .flatten()
            .map(|c| {
                let position = cgmath::Vector3 {
                    x: self.origin.x as f32 + c.origin_x as f32,
                    y: self.origin.y as f32 + c.origin_y as f32,
                    z: BOTTOM_DEPTH as f32 + c.origin_z as f32,
                };

                let rotation = cgmath::Quaternion::from_axis_angle(
                    cgmath::Vector3::unit_z(),
                    cgmath::Deg(0.0),
                );
                let scale = 0.5;

                model::RenderInstance {
                    position,
                    rotation,
                    scale,
                    //TODO: faster if we can use the static strings everywhere
                    label: c.block_type.tex_label().to_string(),
                }
            })
            .collect::<Vec<_>>()
    }

    pub fn gen_default_chunk(origin: Point2<i32>) -> Self {
        debug!("Generating new chunk at ({:?}", origin);
        let solid_fill_height = 10;

        let mut chunk = Chunk {
            origin,
            blocks: [[[None; CHUNK_WIDTH]; CHUNK_WIDTH]; CHUNK_HEIGHT],
        };

        for i in 0..CHUNK_WIDTH {
            for j in 0..CHUNK_WIDTH {
                let block_type = if (i + j) % 2 == 0 {
                    BlockType::Dirt
                } else {
                    BlockType::Weird
                };

                for k in 0..solid_fill_height {
                    chunk.blocks[k][j][i] = Some(Block {
                        block_type,
                        origin_x: i as i32,
                        origin_y: j as i32,
                        origin_z: k as i32,
                    });
                }

                for k in solid_fill_height..solid_fill_height + 3 {
                    // now do some random scattering of blocks on the next row up
                    if rand::random_ratio(4, 10) && chunk.blocks[k - 1][j][i].is_some() {
                        chunk.blocks[k][j][i] = Some(Block {
                            block_type,
                            origin_x: i as i32,
                            origin_y: j as i32,
                            origin_z: k as i32,
                        });
                    }
                }
            }
        }

        chunk
    }
}

pub struct ChunkManagerConfig {
    gen_dist: u32,
    render_dist: u32,
}

impl Default for ChunkManagerConfig {
    fn default() -> Self {
        Self {
            gen_dist: 1,
            render_dist: 1,
        }
    }
}

#[derive(Default)]
pub struct ChunkManager {
    pub chunks: HashMap<Point2<i32>, Chunk>,
    render_keys: HashSet<Point2<i32>>,
    pub config: ChunkManagerConfig,
}

impl ChunkManager {
    pub fn update(&mut self, camera: &camera::Camera, projection: &camera::Projection) {
        // First, check if we need to gen any new chunks
        let new_gen_chunks =
            gen_chunk_origins_near_player(camera.position, self.config.gen_dist as i32)
                .into_iter()
                .filter(|x| !self.chunks.contains_key(x))
                .collect::<Vec<_>>();

        // Gen any new chunks
        for new_origin in new_gen_chunks {
            self.chunks
                .insert(new_origin, Chunk::gen_default_chunk(new_origin));
        }

        // now update the renderable chunks
        self.render_keys =
            gen_chunk_origins_near_player(camera.position, self.config.render_dist as i32)
                .into_iter()
                .filter(|x| self.chunks.contains_key(x))
                .filter(|x| {
                    in_camera_view(
                        camera,
                        projection.fovy.0,
                        self.chunks.get(x).unwrap().origin,
                    )
                })
                .collect();
    }

    pub fn gen_instances(&self) -> Vec<model::RenderInstance> {
        let mut instances = Vec::new();

        for k in &self.render_keys {
            if let Some(chunk) = self.chunks.get(k) {
                instances.append(&mut chunk.gen_instances());
            }
        }

        instances
    }
}

fn gen_chunk_origins_near_player(
    player_pos: cgmath::Point3<f32>,
    dist: i32,
) -> HashSet<Point2<i32>> {
    // Draws a circle around the player, and returns all the chunk origins in this
    // circle. This can be used to calculate which chunks should be rendered,
    // or which new chunks should be generated.

    let mut origins = HashSet::new();

    // First, gen all the candidates in the possible square
    let min_x = player_pos.x as i32 - (CHUNK_WIDTH as i32 * dist);
    let max_x = player_pos.x as i32 + (CHUNK_WIDTH as i32 * dist);
    let start_x = lowest_multiple_above(CHUNK_WIDTH as i32, min_x);
    let min_y = player_pos.y as i32 - (CHUNK_WIDTH as i32 * dist);
    let max_y = player_pos.y as i32 + (CHUNK_WIDTH as i32 * dist);
    let start_y = lowest_multiple_above(CHUNK_WIDTH as i32, min_y);

    let check_dist = CHUNK_WIDTH as f32 * dist as f32;
    let center_offset = CHUNK_WIDTH as f32 / 2.0;

    dbg!(min_x);
    dbg!(max_x);
    dbg!(start_x);
    dbg!(min_y);
    dbg!(max_y);
    dbg!(start_y);
    dbg!(check_dist);
    dbg!(center_offset);

    for origin_x in (start_x..max_x).step_by(CHUNK_WIDTH) {
        for origin_y in (start_y..max_y).step_by(CHUNK_WIDTH) {
            // check for the distances
            if Vector2::new(
                player_pos.x - origin_x as f32 - center_offset,
                player_pos.y - origin_y as f32 - center_offset,
            )
            .magnitude()
                < check_dist
            {
                origins.insert(Point2::new(origin_x, origin_y));
            }
        }
    }

    origins
}

fn lowest_multiple_above(x: i32, n: i32) -> i32 {
    //TODO: might need to optimise this to be branchless
    if n % x == 0 {
        return n;
    }

    if x.signum() == n.signum() {
        ((n / x) + 1) * x
    } else {
        // if backwards, actually want to find
        // the largest of x.abs() which is less
        // than n.abs()
        (n / x) * x
    }
}

fn in_camera_view(camera: &camera::Camera, fov: f32, chunk_origin: Point2<i32>) -> bool {
    let corners = vec![
        Point2::new(chunk_origin.x, chunk_origin.y),
        Point2::new(chunk_origin.x, chunk_origin.y + CHUNK_WIDTH as i32),
        Point2::new(chunk_origin.x + CHUNK_WIDTH as i32, chunk_origin.y),
        Point2::new(
            chunk_origin.x + CHUNK_WIDTH as i32,
            chunk_origin.y + CHUNK_WIDTH as i32,
        ),
    ];

    let camera_pos = Point2::new(camera.position.x as i32, camera.position.y as i32);
    let forward = Vector2::new(camera.yaw.cos(), camera.yaw.sin());

    for c in corners {
        // have to transform the vertex
        let to_corner = (c - camera_pos).cast::<f32>().unwrap();
        let dir = to_corner.normalize();
        let angle_cos = forward.dot(dir);
        let max_angle_cos = (fov / 2.0).cos();

        if angle_cos >= max_angle_cos {
            return true;
        }
    }

    false
}

#[cfg(test)]
mod tests {
    use cgmath::{Point3, Rad};

    use crate::camera::Camera;

    use super::*;

    #[test]
    fn test_in_camera_view() {
        let camera = Camera::new([0.0, 0.0, 0.0], Rad(0.0), Rad(0.0));
        let fov = Rad(45.0);

        assert_eq!(in_camera_view(&camera, fov.0, Point2::new(0, 0)), true);
        assert_eq!(in_camera_view(&camera, fov.0, Point2::new(50, 50)), false);
        assert_eq!(in_camera_view(&camera, fov.0, Point2::new(-50, 50)), false);
        assert_eq!(in_camera_view(&camera, fov.0, Point2::new(-50, -50)), false);
    }

    #[test]
    fn test_lowest_multiple_above() {
        let cases = vec![
            (1, 0, 0),
            (3, 7, 9),
            (2, 8, 8),
            (8, 2, 8),
            (-3, -5, -6),
            (3, -5, -3),
            (-3, 5, 3),
        ];

        for (x, n, res) in cases {
            assert_eq!(lowest_multiple_above(x, n), res)
        }
    }

    #[test]
    fn test_gen_origins_near_player() {
        let cases = vec![
            (
                Point3::new(CHUNK_WIDTH as f32 / 2.0, CHUNK_WIDTH as f32 / 2.0, 0.0),
                1,
                HashSet::from_iter(vec![Point2::new(0, 0)]),
            ),
            (
                Point3::new(CHUNK_WIDTH as f32 / 2.0, CHUNK_WIDTH as f32 / 2.0, 0.0),
                2,
                HashSet::from_iter(vec![
                    Point2::new(0, 0),
                    Point2::new(-16, 0),
                    Point2::new(16, 0),
                    Point2::new(0, -16),
                    Point2::new(0, 16),
                    Point2::new(-16, -16),
                    Point2::new(-16, 16),
                    Point2::new(16, -16),
                    Point2::new(16, 16),
                ]),
            ),
        ];

        for (player_pos, dist, res) in cases {
            assert_eq!(gen_chunk_origins_near_player(player_pos, dist), res);
        }
    }
}
