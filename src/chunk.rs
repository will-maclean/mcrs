use std::collections::HashMap;

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
    origin_x: i32,
    origin_y: i32,
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
                    x: self.origin_x as f32 + c.origin_x as f32,
                    y: self.origin_y as f32 + c.origin_y as f32,
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

    pub fn gen_default_chunk(origin_x: i32, origin_y: i32) -> Self {
        debug!("Generating new chunk at ({origin_x}, {origin_y}");
        let solid_fill_height = 10;

        let mut chunk = Chunk {
            origin_x,
            origin_y,
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
    pub chunks: HashMap<(i32, i32), Chunk>,
    render_keys: Vec<(i32, i32)>,
    pub config: ChunkManagerConfig,
}

impl ChunkManager {
    pub fn update(&mut self, camera: &camera::Camera, projection: &camera::Projection) {
        // First, check if we need to gen any new chunks
        let new_gen_chunks =
            Self::gen_chunk_origins_near_player(camera.position, self.config.gen_dist as i32)
                .into_iter()
                .filter(|x| !self.chunks.contains_key(x))
                .collect::<Vec<_>>();

        // Gen any new chunks
        for (new_origin_x, new_origin_y) in new_gen_chunks {
            self.chunks.insert(
                (new_origin_x, new_origin_y),
                Chunk::gen_default_chunk(new_origin_x, new_origin_y),
            );
        }

        // now update the renderable chunks
        self.render_keys =
            Self::gen_chunk_origins_near_player(camera.position, self.config.render_dist as i32)
                .into_iter()
                .filter(|x| self.chunks.contains_key(x))
                .filter(|x| in_camera_view(camera, projection.fovy.0, self.chunks.get(x).unwrap()))
                .collect::<Vec<(i32, i32)>>();
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

    fn gen_chunk_origins_near_player(
        player_pos: cgmath::Point3<f32>,
        dist: i32,
    ) -> Vec<(i32, i32)> {
        // Draws a circle around the player, and returns all the chunk origins in this
        // circle. This can be used to calculate which chunks should be rendered,
        // or which new chunks should be generated.

        let mut origins = Vec::new();

        // First, gen all the candidates in the possible square
        let min_x = player_pos.x as i32 - (CHUNK_WIDTH as i32 * dist);
        let max_x = player_pos.x as i32 + (CHUNK_WIDTH as i32 * dist);
        let start_x = Self::lowest_multiple_above(CHUNK_WIDTH as u32, min_x as u32) as i32;
        let min_y = player_pos.y as i32 - (CHUNK_WIDTH as i32 * dist);
        let max_y = player_pos.y as i32 + (CHUNK_WIDTH as i32 * dist);
        let start_y = Self::lowest_multiple_above(CHUNK_WIDTH as u32, min_y as u32) as i32;

        let check_dist = CHUNK_WIDTH as f32 * dist as f32;
        let center_offset = CHUNK_WIDTH as f32 / 2.0;

        for origin_x in (start_x..max_x).step_by(CHUNK_WIDTH) {
            for origin_y in (start_y..max_y).step_by(CHUNK_WIDTH) {
                // check for the distances
                if Vector2::new(
                    player_pos.x - origin_x as f32 + center_offset,
                    player_pos.y - origin_y as f32 + center_offset,
                )
                .magnitude()
                    < check_dist
                {
                    origins.push((origin_x, origin_y));
                }
            }
        }

        origins
    }

    fn lowest_multiple_above(x: u32, n: u32) -> u32 {
        ((n / x) + 1) * x
    }
}

fn in_camera_view(camera: &camera::Camera, fov: f32, chunk: &Chunk) -> bool {
    let corners = vec![
        Point2::new(chunk.origin_x, chunk.origin_y),
        Point2::new(chunk.origin_x, chunk.origin_y + CHUNK_WIDTH as i32),
        Point2::new(chunk.origin_x + CHUNK_WIDTH as i32, chunk.origin_y),
        Point2::new(
            chunk.origin_x + CHUNK_WIDTH as i32,
            chunk.origin_y + CHUNK_WIDTH as i32,
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
