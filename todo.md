# TODO

- [x] Model support
    - [x] Obj models
- [ ] Textures
    - [x] Texture Manager
    - [x] Block textures (different texture per side)
        - Can do this using texture arrays in the shader. Mostly works as
            normal, except i index into the texture array when rendering
            a particular instance/texture. Shouldn't be too hard to set
            up, but the number of texture indices to track will become
            hard to do manually - will need the texture manager to handle
            the indices.
    - [x] Obj textures
    - [ ] Different textures on different sides of the one bloc
- [ ] Coord/Camera cleanup
    - [ ] Movement not aligned?
- [ ] Chunks
    - [ ] Chunk Manager
        - [ ] Stores chunks
        - [ ] Trigger chunk gen
        - [ ] Controls when to render
    - [ ] Gen
        - [ ] Use noise in height maps
        - [ ] biomes
- [ ] Blocks
    - [ ] Add different block types
- [ ] QOL
    - [ ] Debug screen
        - [ ] Axis wireframe render
        - [x] Quick text (location, view, etc.)
    - [ ] Logging
- Code refactor
    - [ ] Strip out of state
    - [x] Move out of lib
    - [ ] Build a real game loop
