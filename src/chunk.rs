use crate::model;
use cgmath::prelude::*;

const CHUNK_WIDTH: usize = 16;
const CHUNK_HEIGHT: usize = 256;
const BOTTOM_DEPTH: i32 = -128;

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
    //FIXME: render should be in the chunk mnager somewhere, not the chunk itself
    render: bool,
}

impl Chunk {
    pub fn gen_instances(&self) -> Vec<model::RenderInstance> {
        self.blocks
            .iter()
            .flatten()
            .into_iter()
            .flatten()
            .into_iter()
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
        let solid_fill_height = 126;

        let mut chunk = Chunk {
            origin_x,
            origin_y,
            blocks: [[[None; CHUNK_WIDTH]; CHUNK_WIDTH]; CHUNK_HEIGHT],
            render: true,
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
                    if rand::random_ratio(4, 10) {
                        if let Some(_) = chunk.blocks[k - 1][j][i] {
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
            gen_dist: 4,
            render_dist: 3,
        }
    }
}

#[derive(Default)]
pub struct ChunkManager {
    //TODO: probably a more efficient type for this storage
    pub chunks: Vec<Chunk>,
    pub config: ChunkManagerConfig,
}

impl ChunkManager {
    pub fn update(&mut self, player_pos: cgmath::Point3<f32>) {}

    pub fn gen_instances(&self) -> Vec<model::RenderInstance> {
        let mut instances = Vec::new();

        for chunk in &self.chunks {
            if chunk.render {
                instances.append(&mut chunk.gen_instances());
            }
        }

        instances
    }
}
