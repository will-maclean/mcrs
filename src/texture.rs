use std::collections::HashMap;

use image::GenericImageView;

pub struct DepthTexture {
    pub texture: wgpu::Texture,
    pub view: wgpu::TextureView,
    pub sampler: wgpu::Sampler,
}

impl DepthTexture {
    pub const DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;

    pub fn new(device: &wgpu::Device, config: &wgpu::SurfaceConfiguration, label: &str) -> Self {
        let size = wgpu::Extent3d {
            // 2.
            width: config.width.max(1),
            height: config.height.max(1),
            depth_or_array_layers: 1,
        };
        let desc = wgpu::TextureDescriptor {
            label: Some(label),
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: Self::DEPTH_FORMAT,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT // 3.
                | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
            size,
        };
        let texture = device.create_texture(&desc);
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            // 4.
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Nearest,
            compare: Some(wgpu::CompareFunction::LessEqual), // 5.
            lod_min_clamp: 0.0,
            lod_max_clamp: 100.0,
            ..Default::default()
        });

        Self {
            texture,
            view,
            sampler,
        }
    }
}

pub struct Texture {
    raw: Vec<u8>,
    width: u32,
    height: u32,
    label: String,
}

impl Texture {
    pub fn from_image(label: &str, img: &image::DynamicImage) -> Self {
        let rgba = img.to_rgba8();
        let dimensions = img.dimensions();

        Self {
            raw: rgba.into_vec(),
            width: dimensions.0,
            height: dimensions.1,
            label: label.to_string(),
        }
    }
}

// We need to load all the textures onto the GPU at once. So,
// we can ensure that all textures are prepared ahead of time
// with a builder. Add all textures to the builder at the start
// of the program, then convert it to a TextureManager for
// actual use.
pub struct TextureManagerBuilder {
    map: HashMap<String, Texture>,
    width: Option<u32>,
    height: Option<u32>,
}

impl TextureManagerBuilder {
    pub fn new(width: Option<u32>, height: Option<u32>) -> Self {
        Self {
            map: Default::default(),
            width,
            height,
        }
    }

    pub fn add_texture(&mut self, name: &str, texture: Texture) {
        if let Some(width) = self.width {
            if let Some(height) = self.height {
                if texture.width != width || texture.height != height {
                    //TODO: better error handling
                    panic!("Mismatched texture shapes");
                }
            }
        } else {
            self.width = Some(texture.width);
            self.height = Some(texture.height);
        }
        let _ = self.map.insert(name.to_string(), texture);
    }
}

struct TMVal {
    //TODO: Don't know if there's any reason to store the textures once the
    // setup is complete
    texture: Texture,
    index: usize,
}

#[derive(Default)]
pub struct TextureManager {
    map: HashMap<String, TMVal>,
    width: u32,
    height: u32,
}

impl From<TextureManagerBuilder> for TextureManager {
    fn from(value: TextureManagerBuilder) -> Self {
        let mut map = HashMap::default();
        let mut index = 0;

        for (k, v) in value.map {
            map.insert(k, TMVal { texture: v, index });
            index += 1;
        }

        Self {
            map,
            width: value.width.unwrap(),
            height: value.height.unwrap(),
        }
    }
}

impl TextureManager {
    pub const DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;
    pub fn create_and_submit_texture_array(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) -> (wgpu::BindGroup, wgpu::BindGroupLayout) {
        let texture_array = device.create_texture(&wgpu::TextureDescriptor {
            size: wgpu::Extent3d {
                width: self.width,
                height: self.height,
                depth_or_array_layers: self.map.keys().len() as u32,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            label: Some("texture_array"),
            view_formats: &[],
        });

        let texture_array_view = texture_array.create_view(&wgpu::TextureViewDescriptor {
            dimension: Some(wgpu::TextureViewDimension::D2Array),
            ..Default::default()
        });

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        let bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2Array,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    count: None,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                },
            ],
            label: Some("texture_bind_group_layout"),
        });

        let bg = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &bgl,
            label: Some("texture_bind_group"),
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&texture_array_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
        });

        for entry in self.map.values() {
            self.add_individual_texture(entry, queue, &texture_array);
        }

        (bg, bgl)
    }

    fn add_individual_texture(&self, entry: &TMVal, queue: &wgpu::Queue, tex_arr: &wgpu::Texture) {
        queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: tex_arr,
                mip_level: 0,
                origin: wgpu::Origin3d {
                    x: 0,
                    y: 0,
                    z: entry.index as u32,
                },
                aspect: wgpu::TextureAspect::All,
            },
            &entry.texture.raw,
            wgpu::TexelCopyBufferLayout {
                bytes_per_row: Some(4 * self.width),
                rows_per_image: Some(self.height),
                offset: 0,
            },
            wgpu::Extent3d {
                width: self.width,
                height: self.height,
                depth_or_array_layers: 1,
            },
        );
    }

    pub fn lookup_idx(&self, key: &str) -> Option<usize> {
        if self.map.contains_key(key) {
            return Some(self.map.get(key).unwrap().index);
        }

        None
    }
}
