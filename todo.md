# TODO

- [x] Model support
    - [x] Obj models
- [ ] Textures
    - [x] Texture Manager
    - [x] Block textures (different texture per side)
    - [x] Obj textures
    - [ ] Different textures on different sides of the one bloc
- [x] Coord/Camera cleanup
    - [x] Movement not aligned?
- [ ] Chunks
    - [ ] Chunk Manager
        - [x] Stores chunks
        - [x] Trigger chunk gen
        - [x] Controls when to render
        - [ ] Update block visibility
    - [ ] Gen
        - [ ] Use noise in height maps
        - [ ] biomes
        - [ ] Link to neighbouring chunks
- [ ] Blocks
    - [ ] Add different block types
    - [ ] Add characteristics (how? struct attrs? component/trait type?)
- [ ] QOL
    - [ ] Debug screen
        - [ ] Axis wireframe render
        - [ ] FPS
        - [x] Quick text (location, view, etc.)
    - [x] Logging
- Code refactor
    - [ ] Strip out of state -> into game::MCRS??
    - [x] Move out of lib
    - [x] Build a real game loop
- Rendering
    - [ ] Lighting
- Performance
    - [ ] Instance Culling
        - Now that we can gen new chunks, we can easily have 10k+ instances
        in our render range. That means we need to start being performant
        - Few different spots where we can cull:
            - [x] At a chunk level -> frustum culling
                - Is this actually working?? Seems like too many instances being rendered
            - At a block (instance) level -> frustum culling, backface culling, occlusion culling
            3. More advanced (GPU-side) culls
        - Can any of this be vectorised??
- Gameplay
    - [ ] Place/break blocks
        - [ ] raycasting
            - API for interacting with blocks is nearly done
            - need to flesh out api
            - model is to return a rayresult object that can then be operated on
                - place and remove block still required
            - can then do something like:
```rust 
let ray = Ray::from_camera(&camera);
let rayres = chunk_manager.cast_ray(ray);
match rayres {
    RayResult::Block(block_loc, face, is_interactable) => {
        match action {
            LeftClick => {
                chunk_manager.break_block(block_loc);
            },
            RightClick => {
                // trigger some sort of interation if the 
                // block is interactible, place block
                // otherwise
                if is_interactable {
                    // interact
                } else {
                    let block = Block::new();
                    chunk_manager.place_block_against(block_loc, face);
                }
            }
        }
    }
}
```

    - [ ] HUD
    - [ ] Inventory
    - [ ] Player instead of camera
        - [ ] Gravity
        - [ ] Collision detection
