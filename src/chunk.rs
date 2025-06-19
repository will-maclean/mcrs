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
    Stone,
}

impl BlockType {
    fn tex_label(&self) -> &'static str {
        match self {
            Self::Dirt => "dirt",
            Self::Stone => "stone",
        }
    }
}

#[derive(Debug, Clone, Copy)]
enum BlockExposure {
    Internal,
    External,
    ChunkBorder,
}

#[derive(Debug, Clone, Copy)]
struct Block {
    origin_x: i32,
    origin_y: i32,
    origin_z: i32,
    block_type: BlockType,

    exposure: BlockExposure,
}

impl Block {
    fn visible(&self) -> bool {
        //TODO: this is extremely basic, but might
        //be good enough to keep us going for a while
        match self.exposure {
            BlockExposure::ChunkBorder => false,
            BlockExposure::Internal => false,
            BlockExposure::External => true,
        }
    }
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
            .filter(|b| b.visible())
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
                for k in 0..solid_fill_height {
                    let block_type = if k == solid_fill_height - 1 {
                        BlockType::Stone
                    } else {
                        BlockType::Stone
                    };

                    let exposure = if k == solid_fill_height - 1 {
                        BlockExposure::External
                    } else if i == 0 || i == CHUNK_WIDTH - 1 || j == 0 || j == CHUNK_WIDTH - 1 {
                        BlockExposure::ChunkBorder
                    } else {
                        BlockExposure::Internal
                    };
                    chunk.blocks[k][j][i] = Some(Block {
                        origin_x: i as i32,
                        origin_y: j as i32,
                        origin_z: k as i32,
                        block_type,
                        exposure,
                    });
                }

                for k in solid_fill_height..solid_fill_height + 3 {
                    // now do some random scattering of blocks on the next row up
                    if rand::random_ratio(4, 10) && chunk.blocks[k - 1][j][i].is_some() {
                        chunk.blocks[k][j][i] = Some(Block {
                            origin_x: i as i32,
                            origin_y: j as i32,
                            origin_z: k as i32,
                            block_type: BlockType::Dirt,
                            exposure: BlockExposure::External,
                        });
                    }
                }
            }
        }

        chunk
    }

    fn update_exposure(&mut self) {
        //NOTE: should only be called when required, not every tick (if can be avoided)
        todo!()
    }
}

pub struct ChunkManagerConfig {
    gen_dist: u32,
    render_dist: u32,
}

impl Default for ChunkManagerConfig {
    fn default() -> Self {
        Self {
            gen_dist: 2,
            render_dist: 2,
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
                    in_camera_view(camera, projection.fovy, self.chunks.get(x).unwrap().origin)
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

        debug!(
            "ChunkManager submitting {} chunks, with {} total instances to render",
            self.render_keys.len(),
            instances.len()
        );

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

fn in_camera_view(
    camera: &camera::Camera,
    fov: cgmath::Rad<f32>,
    chunk_origin: Point2<i32>,
) -> bool {
    if camera.position.x >= chunk_origin.x as f32
        && camera.position.x <= chunk_origin.x as f32 + CHUNK_WIDTH as f32
        && camera.position.y >= chunk_origin.y as f32
        && camera.position.y <= chunk_origin.y as f32 + CHUNK_WIDTH as f32
    {
        return true;
    }
    let corners = vec![
        Vector2::new(chunk_origin.x, chunk_origin.y),
        Vector2::new(chunk_origin.x, chunk_origin.y + CHUNK_WIDTH as i32),
        Vector2::new(chunk_origin.x + CHUNK_WIDTH as i32, chunk_origin.y),
        Vector2::new(
            chunk_origin.x + CHUNK_WIDTH as i32,
            chunk_origin.y + CHUNK_WIDTH as i32,
        ),
    ];

    let camera_pos = Vector2::new(camera.position.x as f32, camera.position.y as f32);
    let forward = Vector2::new(camera.yaw.cos(), camera.yaw.sin());

    for c in corners {
        let c = c.cast::<f32>().unwrap();
        let view = c - camera_pos;
        let angle = view.angle(forward);
        // have to transform the vertex
        if angle.0.abs() < fov.0 {
            return true;
        }
    }

    false
}

#[cfg(test)]
mod tests {
    use cgmath::{Deg, Point3, Rad};

    use crate::camera::Camera;

    use super::*;

    #[test]
    fn test_in_camera_view() {
        let camera = Camera::new([0.0, 0.0, 0.0], Rad(0.0), Rad(0.0));
        let fov = Deg(45.0);

        assert_eq!(in_camera_view(&camera, fov.into(), Point2::new(0, 0)), true);
        assert_eq!(
            in_camera_view(&camera, fov.into(), Point2::new(-50, 50)),
            false
        );
        assert_eq!(
            in_camera_view(&camera, fov.into(), Point2::new(-50, -50)),
            false
        );
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
