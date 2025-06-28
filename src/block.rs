use cgmath::Point3;
use strum::EnumIter;

#[derive(Debug, Clone, Copy)]
pub enum BlockType {
    Dirt,
    Stone,
}

impl BlockType {
    pub fn tex_label(&self) -> &'static str {
        match self {
            Self::Dirt => "dirt",
            Self::Stone => "stone",
        }
    }
}

//TODO: remove Copy
#[derive(Debug, Clone, Copy)]
pub struct Block {
    pub block_type: BlockType,
    visible_arr: [bool; 6],
}

impl Block {
    pub fn new(block_type: BlockType) -> Self {
        Self {
            block_type,
            visible_arr: [false; 6],
        }
    }

    pub fn visible(&self, face: BlockFace) -> bool {
        self.visible_arr[face as usize]
    }

    pub fn set_visible(&mut self, face: BlockFace, visibility: bool) {
        self.visible_arr[face as usize] = visibility
    }
}

#[derive(Debug, Clone, Copy, PartialEq, EnumIter)]
pub enum BlockFace {
    XPos = 0,
    XNeg = 1,
    YPos = 2,
    YNeg = 3,
    ZPos = 4,
    ZNeg = 5,
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

pub fn block_contains(block_pos: Point3<i32>, test_pos: Point3<f32>) -> bool {
    let block_pos = block_pos.cast::<f32>().unwrap();

    test_pos.x >= block_pos.x
        && test_pos.x <= (block_pos.x + 1.0)
        && test_pos.y >= block_pos.y
        && test_pos.y <= (block_pos.y + 1.0)
        && test_pos.z >= block_pos.z
        && test_pos.z <= (block_pos.z + 1.0)
}
