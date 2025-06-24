use std::collections::{HashMap, HashSet};

use crate::{
    camera, model,
    raycasting::{BlockFace, Ray, RayResult},
};
use cgmath::{prelude::*, Point2, Point3, Vector2};
use log::debug;

const CHUNK_WIDTH: usize = 16;
const CHUNK_HEIGHT: usize = 256;
const BOTTOM_DEPTH: i32 = -128;

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
pub struct Block {
    block_type: BlockType,
    exposure: BlockExposure,
}

impl Block {
    pub fn new(block_type: BlockType) -> Self {
        Self {
            block_type,
            //FIXME: implement
            exposure: BlockExposure::External,
        }
    }
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
    fn get(&self, loc: Point3<usize>) -> Option<Block> {
        self.blocks[loc.z][loc.y][loc.x]
    }

    fn idx_to_world(&self, x: usize, y: usize, z: usize) -> Point3<i32> {
        Point3::new(
            x as i32 + self.origin.x,
            y as i32 + self.origin.y,
            BOTTOM_DEPTH + z as i32,
        )
    }

    pub fn gen_instances(&self) -> Vec<model::RenderInstance> {
        //FIXME: Surely this can be done nice with some sort of mapping
        let mut result = Vec::new();

        for x in 0..CHUNK_WIDTH {
            for y in 0..CHUNK_WIDTH {
                for z in 0..CHUNK_HEIGHT {
                    if let Some(block) = self.get(Point3::new(x, y, z)) {
                        if block.visible() {
                            let position =
                                self.idx_to_world(x, y, z).cast::<f32>().unwrap().to_vec();

                            let rotation = cgmath::Quaternion::from_axis_angle(
                                cgmath::Vector3::unit_z(),
                                cgmath::Deg(0.0),
                            );
                            let scale = 0.5;

                            result.push(model::RenderInstance {
                                position,
                                rotation,
                                scale,
                                //TODO: faster if we can use the static strings everywhere
                                label: block.block_type.tex_label().to_string(),
                            });
                        }
                    }
                }
            }
        }

        result
    }

    pub fn gen_empty_chunk(origin: Point2<i32>) -> Self {
        // Probably only going to be used for testing but
        // whatever
        Self {
            origin,
            blocks: [[[None; CHUNK_WIDTH]; CHUNK_WIDTH]; CHUNK_HEIGHT],
        }
    }

    pub fn gen_default_chunk(origin: Point2<i32>) -> Self {
        debug!("Generating new chunk at ({:?}", origin);
        let solid_fill_height: usize = (-5 - BOTTOM_DEPTH) as usize;

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
                        block_type,
                        exposure,
                    });
                }

                for k in solid_fill_height..solid_fill_height + 3 {
                    // now do some random scattering of blocks on the next row up
                    if rand::random_ratio(4, 10) && chunk.blocks[k - 1][j][i].is_some() {
                        chunk.blocks[k][j][i] = Some(Block {
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

    pub fn cast_ray(&self, ray: Ray) -> RayResult {
        let iter_dist = ray.max_dist / ray.n_tests as f32;
        for i in 0..ray.n_tests {
            let test_pos_f32 = ray.pos + (iter_dist * i as f32 * ray.dir);
            //TODO: Check that cast works correctly
            let test_pos = test_pos_f32.cast::<i32>().unwrap();
            if let Ok(test_pos_block) = self.world_to_local(test_pos) {
                if let Some(block) = self.get(test_pos_block) {
                    let result = RayResult::Block {
                        loc: test_pos,
                        //TODO: figure out how to do the face detection
                        face: BlockFace::ZPos,
                        dist: test_pos_f32.to_vec().magnitude(),
                    };

                    debug!("{:?}", result);

                    return result;
                }
            }
        }
        RayResult::None
    }

    pub fn mutate_block<F>(&mut self, block_loc: Point3<i32>, f: F)
    where
        F: FnOnce(&mut Option<Block>),
    {
        if let Ok(local_pos) = self.world_to_local(block_loc) {
            let mut block = self.get(local_pos);
            f(&mut block)
        }
    }

    fn world_to_local(&self, pos: Point3<i32>) -> Result<Point3<usize>, ()> {
        let point = Point3::new(
            pos.x - self.origin.x,
            pos.y - self.origin.y,
            pos.z - BOTTOM_DEPTH,
        );

        if point.x < 0
            || point.x >= CHUNK_WIDTH as i32
            || point.y < 0
            || point.y >= CHUNK_WIDTH as i32
            || point.z < 0
            || point.z >= CHUNK_HEIGHT as i32
        {
            Err(())
        } else {
            Ok(point.cast::<usize>().unwrap())
        }
    }

    pub fn set_block(&mut self, loc: Point3<i32>, block: Block) -> Result<(), ()> {
        if let Ok(local_pos) = self.world_to_local(loc) {
            // Can only place in an empty location
            if let Some(_) = self.get(local_pos) {
                Err(())
            } else {
                self.blocks[local_pos.z][local_pos.y][local_pos.x] = Some(block);
                Ok(())
            }
        } else {
            Err(())
        }
    }

    pub fn remove_block(&mut self, loc: Point3<i32>) -> Result<Block, ()> {
        if let Ok(local_pos) = self.world_to_local(loc) {
            // Can only place in an empty location
            if let Some(block) = self.get(local_pos) {
                self.blocks[local_pos.z][local_pos.y][local_pos.x] = None;
                return Ok(block);
            }
        }
        Err(())
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

        instances
    }

    pub fn cast_ray(&self, ray: Ray) -> RayResult {
        //TODO: for now, this will only allow the play to
        //cast rays inside their own chunk. What we really need
        //is to do a ray cast at a chunk level, then iterate throut
        //the results, closest to furthest, looking for a collision
        let chunk_loc = block_to_chunk(ray.pos.cast::<i32>().unwrap());
        if let Some(chunk) = self.chunks.get(&chunk_loc) {
            chunk.cast_ray(ray)
        } else {
            RayResult::None
        }
    }

    pub fn mutate_block<F>(&mut self, block_loc: Point3<i32>, f: F)
    where
        F: FnOnce(&mut Option<Block>),
    {
        //TODO: smarter return codes?

        let chunk_loc = block_to_chunk(block_loc);
        if let Some(chunk) = self.chunks.get_mut(&chunk_loc) {
            chunk.mutate_block(block_loc, f)
        }
    }

    pub fn set_block(&mut self, loc: Point3<i32>, block: Block) -> Result<(), ()> {
        let chunk_loc = block_to_chunk(loc);
        if let Some(chunk) = self.chunks.get_mut(&chunk_loc) {
            chunk.set_block(loc, block)
        } else {
            Err(())
        }
    }

    pub fn remove_block(&mut self, loc: Point3<i32>) -> Result<Block, ()> {
        let chunk_loc = block_to_chunk(loc);
        if let Some(chunk) = self.chunks.get_mut(&chunk_loc) {
            chunk.remove_block(loc)
        } else {
            Err(())
        }
    }
}

fn block_to_chunk(block_pos: Point3<i32>) -> Point2<i32> {
    Point2 {
        x: block_pos.x / CHUNK_WIDTH as i32,
        y: block_pos.y / CHUNK_WIDTH as i32,
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

    #[test]
    fn test_chunk_raycasting() {
        let camera = Camera::new(Point3::new(1.0, 1.0, 1.0), Rad(0.0), Rad(0.0));
        let chunk_origin = Point2::new(0, 0);
        let mut chunk = Chunk::gen_empty_chunk(chunk_origin);

        // now, the chunk is empty, so casting a ray now
        // should return a None
        assert_eq!(chunk.cast_ray(Ray::from(&camera)), RayResult::None);

        // insert a block that the camera SHOULD be able to see
        let block = Block::new(BlockType::Dirt);
        let block_pos = Point3::new(2, 1, 1);
        let _ = chunk.set_block(block_pos, block);
        if let RayResult::Block { loc, .. } = chunk.cast_ray(Ray::from(&camera)) {
            assert_eq!(loc, block_pos);
        } else {
            assert!(false);
        }

        // now insert a block that camera ray SHOULDN'T hit
        let _ = chunk.remove_block(block_pos);
        let block_pos = Point3::new(1, 2, 1);
        let _ = chunk.set_block(block_pos, block);
        assert_eq!(chunk.cast_ray(Ray::from(&camera)), RayResult::None);
    }

    #[test]
    fn test_chunk_insert_remove() {
        let mut chunk = Chunk::gen_empty_chunk(Point2::new(0, 0));

        // check the chunk is currently empty
        for x in 0..CHUNK_WIDTH {
            for y in 0..CHUNK_WIDTH {
                for z in 0..CHUNK_HEIGHT {
                    if let Ok(_) = chunk.remove_block(Point3::new(x as i32, y as i32, z as i32)) {
                        assert!(false);
                    }
                }
            }
        }

        // test insert
        let pos = Point3::new(1, 2, 3);
        if let Err(_) = chunk.set_block(pos, Block::new(BlockType::Dirt)) {
            assert!(false, "set block failed");
        }

        // test remove
        if let Err(_) = chunk.remove_block(pos) {
            assert!(false, "remove block failed");
        }
    }

    #[test]
    fn test_world_to_local() {
        let chunk = Chunk::gen_empty_chunk(Point2::new(0, 0));

        let test_pos = Point3::new(1, 2, 3);
        let chunk_coords = chunk.world_to_local(test_pos).unwrap();
        assert_eq!(chunk_coords, Point3::new(1, 2, (3 - BOTTOM_DEPTH) as usize));
    }
}
