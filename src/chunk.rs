use std::collections::{HashMap, HashSet};

use crate::{
    block::{Block, BlockFace, BlockType},
    camera, model,
    raycasting::{get_colliding_face, Ray, RayResult},
};
use cgmath::{prelude::*, Point2, Point3, Vector2};
use log::debug;
use strum::IntoEnumIterator;

const CHUNK_WIDTH: usize = 16;
const CHUNK_HEIGHT: usize = 256;
const BOTTOM_DEPTH: i32 = -128;

#[derive(Debug, Copy, Clone)]
pub enum ChunkCoord {
    Local(Point3<usize>),
    World(Point3<i32>),
}

impl From<Point3<f32>> for ChunkCoord {
    fn from(value: Point3<f32>) -> Self {
        chunk_coord_global(
            value.x.floor() as i32,
            value.y.floor() as i32,
            value.z.floor() as i32,
        )
    }
}

impl From<Point3<i32>> for ChunkCoord {
    fn from(value: Point3<i32>) -> Self {
        ChunkCoord::World(value)
    }
}

impl ChunkCoord {
    pub fn to_local(self, chunk_origin: Point2<i32>) -> Result<Point3<usize>, ()> {
        match self {
            ChunkCoord::Local(pos) => Ok(pos),
            ChunkCoord::World(pos) => {
                let point = Point3::new(
                    pos.x - chunk_origin.x,
                    pos.y - chunk_origin.y,
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
        }
    }
    pub fn to_world(self, chunk_origin: Point2<i32>) -> Point3<i32> {
        match self {
            ChunkCoord::Local(pos) => Point3::new(
                chunk_origin.x + pos.x as i32,
                chunk_origin.y + pos.y as i32,
                pos.z as i32 + BOTTOM_DEPTH,
            ),
            ChunkCoord::World(pos) => pos,
        }
    }
}

pub fn chunk_coord_local(x: usize, y: usize, z: usize) -> ChunkCoord {
    ChunkCoord::Local(Point3::new(x, y, z))
}

pub fn chunk_coord_global(x: i32, y: i32, z: i32) -> ChunkCoord {
    ChunkCoord::World(Point3::new(x, y, z))
}

#[derive(Debug, Clone)]
pub struct Chunk {
    origin: Point2<i32>,
    blocks: [[[Option<Block>; CHUNK_WIDTH]; CHUNK_WIDTH]; CHUNK_HEIGHT],
}

impl Chunk {

    fn get_ref(&self, loc: ChunkCoord) -> Result<&Option<Block>, ()> {
        if let Ok(local_loc) = loc.to_local(self.origin) {
            Ok(&self.blocks[local_loc.z][local_loc.y][local_loc.x])
        } else {
            Err(())
        }
    }
    
    fn get_ref_mut(&mut self, loc: ChunkCoord) -> Result<&mut Option<Block>, ()> {
        if let Ok(local_loc) = loc.to_local(self.origin) {
            Ok(&mut self.blocks[local_loc.z][local_loc.y][local_loc.x])
        } else {
            Err(())
        }
    }


    fn get(&self, loc: ChunkCoord) -> Result<Option<Block>, ()> {
        if let Ok(local_loc) = loc.to_local(self.origin) {
            Ok(self.blocks[local_loc.z][local_loc.y][local_loc.x])
        } else {
            Err(())
        }
    }

    pub fn gen_instances(&self) -> Vec<model::RenderInstance> {
        //FIXME: Surely this can be done nice with some sort of mapping
        let mut result = Vec::new();

        for x in 0..CHUNK_WIDTH {
            for y in 0..CHUNK_WIDTH {
                for z in 0..CHUNK_HEIGHT {
                    if let Some(block) = self.get(chunk_coord_local(x, y, z)).unwrap() {
                        for face in BlockFace::iter() {
                            if block.visible(face) {
                                let position = chunk_coord_local(x, y, z)
                                    .to_world(self.origin)
                                    .cast::<f32>()
                                    .unwrap()
                                    .to_vec();

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
                                    face,
                                });
                            }
                        }
                    }
                }
            }
        }

        debug!(
            "ChunkManager submitting {} instances (faces) to render",
            result.len()
        );

        result
    }

    // Probably only going to be used for testing
    #[allow(dead_code)]
    pub fn gen_empty_chunk(origin: Point2<i32>) -> Self {
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

                    chunk.blocks[k][j][i] = Some(Block::new(block_type));
                }

                for k in solid_fill_height..solid_fill_height + 3 {
                    // now do some random scattering of blocks on the next row up
                    if rand::random_ratio(4, 10) && chunk.blocks[k - 1][j][i].is_some() {
                        chunk.blocks[k][j][i] = Some(Block::new(BlockType::Dirt));
                    }
                }
            }
        }

        chunk.update_exposure_chunk(None);

        chunk
    }

    pub fn update_exposure_chunk(&mut self, chunk_manager: Option<&ChunkManager>) {
        for x in 0..CHUNK_WIDTH {
            for y in 0..CHUNK_WIDTH {
                for z in 0..CHUNK_HEIGHT {
                    let pos = chunk_coord_local(x, y, z);
                    if let Some(_) = self.get(pos).unwrap() {
                        self.update_exposure_block(pos, chunk_manager);
                    }
                }
            }
        }
    }

    pub fn update_exposure_around(
        &mut self,
        pos: ChunkCoord,
        chunk_manager: Option<&ChunkManager>,
    ) {
        let global_pos = pos.to_world(self.origin);
        for face in BlockFace::iter() {
            let adjacent_block_global = face.adjacent_loc_from(global_pos);
            self.update_exposure_block(ChunkCoord::from(adjacent_block_global), chunk_manager);
        }
    }

    pub fn update_exposure_block(&mut self, pos: ChunkCoord, chunk_manager: Option<&ChunkManager>) {
        // go through and update all the different faces

        // because of the borrow checker, this is a two step process. First, we
        // check the visibility of each face, then we update the block with the
        // visibility information

        let mut visibilities = Vec::new();
        for face in BlockFace::iter() {
            let test_pos =
                ChunkCoord::from(face.adjacent_loc_from(pos.to_world(self.origin)));
            let visible = match test_pos.to_local(self.origin) {
                Ok(_) => {
                    // test_pos is in this chunK
                    !self.block_at(test_pos)
                }
                Err(_) => {
                    // test block is not in this chunk (probably on a chunk boundary). We'll
                    // ask the ChunkManager to check for us
                    match chunk_manager {
                        Some(chunk_manager) => {
                            !chunk_manager.block_at(test_pos.to_world(self.origin))
                        }
                        None => {
                            // no chunk manager provided (maybe hasn't been set up yet). So, if
                            // we're on a chunk boundary, we'll set the
                            // visibility to true

                            true
                        }
                    }
                }
            };

            visibilities.push((face, visible));
        }

        if let Ok(block_ref) = self.get_ref_mut(pos) {
            if let Some(block_ref) = block_ref.as_mut(){
                for (face, visible) in visibilities {
                    block_ref.set_visible(face, visible);
                }
            }
        }
    }

    pub fn cast_ray(&self, ray: Ray) -> RayResult {
        let iter_dist = ray.max_dist / ray.n_tests as f32;
        let iter_ray = ray.dir.normalize() * iter_dist;
        let mut test_pos_f32 = ray.pos.clone();
        for _ in 0..ray.n_tests {
            let test_pos = ChunkCoord::from(test_pos_f32);
            match self.get(test_pos) {
                Ok(get_res) => match get_res {
                    Some(_) => {
                        // there was a collision
                        let result = RayResult::Block {
                            loc: test_pos.to_world(self.origin),
                            face: get_colliding_face(
                                ray,
                                test_pos_f32,
                                test_pos.to_world(self.origin),
                            )
                            .unwrap(),
                            dist: test_pos_f32.to_vec().magnitude(),
                        };

                        debug!("{:?}", result);

                        return result;
                    }
                    None => {}
                },
                Err(_) => {
                    // we've left the current chunk -> assume no ray hits
                    return RayResult::None;
                }
            }

            test_pos_f32 += iter_ray;
        }
        RayResult::None
    }

    pub fn mutate_block<F>(&mut self, block_loc: Point3<i32>, f: F)
    where
        F: FnOnce(&mut Option<Block>),
    {
        match self.get(ChunkCoord::from(block_loc)) {
            Ok(mut block) => f(&mut block),
            Err(_) => {
                // the specified block is not in this chunk
                // not sure how this would come up - I'll leave
                // it as a panic for now to see what scenarios trigger
                // it and how they should be dealt with
                todo!()
            }
        }
    }

    pub fn set_block(&mut self, loc: Point3<i32>, block: Block) -> Result<(), ()> {
        let coord = ChunkCoord::from(loc);
        match self.get(coord) {
            Ok(block_loc) => match block_loc {
                Some(_) => {
                    // Can only place in an empty location
                    Err(())
                }
                None => {
                    let local_coords = coord.to_local(self.origin).unwrap();
                    self.blocks[local_coords.z][local_coords.y][local_coords.x] = Some(block);

                    //TODO: find a way to get the chunk manager passed in here
                    self.update_exposure_block(coord, None);
                    self.update_exposure_around(coord, None);

                    Ok(())
                }
            },
            Err(_) => Err(()),
        }
    }

    pub fn remove_block(&mut self, loc: Point3<i32>) -> Result<Block, ()> {
        let coord = ChunkCoord::from(loc);
        match self.get(coord) {
            Ok(block_loc) => {
                match block_loc {
                    Some(block) => {
                        let local_coords = coord.to_local(self.origin).unwrap();
                        self.blocks[local_coords.z][local_coords.y][local_coords.x] = None;

                        //TODO: find a way to get the chunk manager passed in here
                        self.update_exposure_around(coord, None);

                        Ok(block)
                    }
                    None => {
                        // No block here
                        Err(())
                    }
                }
            }
            Err(_) => {
                // block is not in this chunk
                Err(())
            }
        }
    }

    pub fn block_at(&self, coord: ChunkCoord) -> bool {
        match self.get_ref(coord).unwrap_or(&None) {
            Some(_) => true,
            None => false,
        }
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

    pub fn block_at(&self, loc: Point3<i32>) -> bool {
        let chunk_loc = block_to_chunk(loc);
        if let Some(chunk) = self.chunks.get(&chunk_loc) {
            let coord = ChunkCoord::from(loc);
            if let Some(_) = chunk.get(coord).unwrap_or(None) {
                true
            } else {
                false
            }
        } else {
            false
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
    use cgmath::{Deg, Point3, Rad, Vector3};

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
        let camera = Camera::new(Point3::new(1.0, 1.5, 1.5), Rad(0.0), Rad(0.0));
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
    fn test_vec_cast() {
        // test some vec casts, just for my poor wiltering sanity

        assert_eq!(
            Vector3::new(1.0, 2.0, 3.0).cast::<usize>().unwrap(),
            Vector3::new(1, 2, 3)
        );

        assert_eq!(
            Vector3::new(1.0, -2.0, 3.0).cast::<i32>().unwrap(),
            Vector3::new(1, -2, 3)
        );

        assert_eq!(
            Vector3::new(1.1, 2.9, 3.5).cast::<usize>().unwrap(),
            Vector3::new(1, 2, 3)
        );

        assert_eq!(
            Vector3::new(-1.5, -2.1, -3.9).cast::<i32>().unwrap(),
            Vector3::new(-1, -2, -3)
        );

        // takeaway -> casting from float to i32/usize will NOT round;
        // rather, it clips to the integer portion
    }

    #[test]
    fn test_chunk_coords() {
        let c1 = Chunk::gen_empty_chunk(Point2::new(0, 0));
        let c2 = Chunk::gen_empty_chunk(Point2::new(0, 16));

        // cc1 should be in c1, not c2
        let cc1 = ChunkCoord::from(Point3::new(0, 0, BOTTOM_DEPTH + 1));

        match c1.get(cc1) {
            Ok(block_loc) => match block_loc {
                // block should be empty
                Some(_) => assert!(false),
                None => {}
            },
            // should be in there
            Err(_) => assert!(false),
        }

        match c2.get(cc1) {
            // block shouldb't be here
            Ok(_) => assert!(false),
            Err(_) => {}
        }
    }

    #[test]
    fn test_block_visibility_updates() {
        let mut chunk = Chunk::gen_empty_chunk(Point2::new(0, 0));

        let block_pos1 = Point3::new(0, 0, 0);
        if let Err(_) = chunk.set_block(block_pos1, Block::new(BlockType::Dirt)) {
            assert!(false, "failed to place a block");
        }

        // all faces of the block should be visible

        let block1 = chunk
            .get_ref(ChunkCoord::from(block_pos1))
            .as_ref()
            .unwrap()
            .unwrap();

        for face in BlockFace::iter() {
            assert!(block1.visible(face));
        }

        // now, add a second block
        let block_pos2 = Point3::new(1, 0, 0);
        if let Err(_) = chunk.set_block(block_pos2, Block::new(BlockType::Dirt)) {
            assert!(false, "failed to place a block");
        }

        let block2 = chunk
            .get_ref(ChunkCoord::from(block_pos2))
            .as_ref()
            .unwrap()
            .unwrap();

        // XPos face on block 1 and XNeg face on block two should NOT be visible
        assert_eq!(block1.visible(BlockFace::XPos), false);
        assert_eq!(block2.visible(BlockFace::XNeg), false);
    }
}
