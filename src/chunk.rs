use crate::RenderInstance;
use cgmath::prelude::*;

const CHUNK_WIDTH: usize = 16;
const CHUNK_HEIGHT: usize = 256;
const BOTTOM_DEPTH: i32 = -128;

#[derive(Debug, Clone, Copy)]
pub enum BlockType {
    Dirt,
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
    pub fn gen_instances(&self) -> Vec<RenderInstance> {
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

                let rotation = if position.is_zero() {
                    // this is needed so an object at (0, 0, 0) won't get
                    // scaled to zero. Quaternions can affect scale if they're
                    // not created correctly
                    cgmath::Quaternion::from_axis_angle(cgmath::Vector3::unit_z(), cgmath::Deg(0.0))
                } else {
                    cgmath::Quaternion::from_axis_angle(position.normalize(), cgmath::Deg(0.0))
                };

                RenderInstance { position, rotation }
            })
            .collect::<Vec<_>>()
    }

    pub fn gen_default_chunk(origin_x: i32, origin_y: i32) -> Self {
        let solid_fill_height = 126;

        let mut chunk = Chunk {
            origin_x,
            origin_y,
            blocks: [[[None; CHUNK_WIDTH]; CHUNK_WIDTH]; CHUNK_HEIGHT],
        };

        for i in 0..CHUNK_WIDTH {
            for j in 0..CHUNK_WIDTH {
                for k in 0..solid_fill_height {
                    chunk.blocks[k][j][i] = Some(Block {
                        block_type: BlockType::Dirt,
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
                                block_type: BlockType::Dirt,
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
