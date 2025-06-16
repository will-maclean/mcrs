use std::fs;
use std::sync::Arc;

use log::{debug, info};
use pollster::FutureExt;
use texture::TextureManager;
use wgpu::util::DeviceExt;
use wgpu::{Adapter, Device, Instance, PresentMode, Queue, Surface, SurfaceCapabilities};
use winit::dpi::PhysicalSize;
use winit::event::ElementState;
use winit::event::KeyEvent;
use winit::event::MouseButton;
use winit::event::MouseScrollDelta;
use winit::event::WindowEvent;
use winit::event_loop::EventLoop;
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::Window;

pub mod app;
pub mod camera;
mod chunk;
mod debug_view;
pub mod game;
mod model;
mod player;
mod resources;
mod texture;

use model::Vertex;

pub fn run() {
    info!("Starting MCRS");
    let event_loop = EventLoop::new().unwrap();
    let window_state = app::StateApplication::new();
    let mut game = game::MCRS::new(window_state, event_loop);
    game.run();
}

pub struct State {
    surface: Surface<'static>,
    device: Device,
    queue: Queue,
    config: wgpu::SurfaceConfiguration,
    render_pipeline: wgpu::RenderPipeline,
    camera: camera::Camera,
    camera_uniform: camera::CameraUniform,
    camera_controller: camera::CameraController,
    camera_buffer: wgpu::Buffer,
    camera_bind_group: wgpu::BindGroup,
    projection: camera::Projection,
    instance_buffer: wgpu::Buffer,
    depth_texture: texture::DepthTexture,
    mouse_pressed: bool,
    chunk_manager: chunk::ChunkManager,
    pub debug_view: debug_view::DebugView,
    window: Arc<Window>,
    obj_model: model::Model,
    texture_manager: texture::TextureManager,
    texture_bind_group: wgpu::BindGroup,
    pub running: bool,
    n_instances: usize,
}

impl State {
    pub fn new(window: Window) -> Self {
        let window_arc = Arc::new(window);
        let size = window_arc.inner_size();
        let instance = Self::create_gpu_instance();
        let surface = instance.create_surface(window_arc.clone()).unwrap();
        let adapter = Self::create_adapter(instance, &surface);
        let (device, queue) = Self::create_device(&adapter);
        let surface_caps = surface.get_capabilities(&adapter);
        let config = Self::create_surface_config(size, surface_caps);

        let mut texture_manager_builder = texture::TextureManagerBuilder::new(None, None);
        texture_manager_builder.add_texture(
            "stone",
            texture::Texture::from_image(
                "stone",
                &image::load_from_memory(&fs::read("res/cube-diffuse.jpg").unwrap()).unwrap(),
            ),
        );

        texture_manager_builder.add_texture(
            "weird",
            texture::Texture::from_image(
                "weird",
                &image::load_from_memory(&fs::read("res/cube-normal.png").unwrap()).unwrap(),
            ),
        );

        let texture_manager = TextureManager::from(texture_manager_builder);
        let (texture_bind_group, texture_bind_group_layout) =
            texture_manager.create_and_submit_texture_array(&device, &queue);

        let chunk_manager = chunk::ChunkManager::default();

        surface.configure(&device, &config);

        let depth_texture = texture::DepthTexture::new(&device, &config, "depth_texture");

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
        });

        let (
            camera,
            projection,
            camera_uniform,
            camera_buffer,
            camera_bind_group,
            camera_bind_group_layout,
        ) = Self::setup_camera(&device, &config);

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[&texture_bind_group_layout, &camera_bind_group_layout],
                push_constant_ranges: &[],
            });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[model::ModelVertex::desc(), model::RenderInstanceRaw::desc()],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Cw,
                cull_mode: Some(wgpu::Face::Back),
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: texture::DepthTexture::DEPTH_FORMAT,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
            cache: None,
        });

        let instance_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Instance Buffer"),
            contents: &[],
            usage: wgpu::BufferUsages::VERTEX,
        });

        let obj_model =
            pollster::block_on(resources::load_model("simple_cube.obj", &device)).unwrap();

        let debug_view =
            debug_view::DebugView::new(&device, &config, &queue, window_arc.scale_factor());

        Self {
            chunk_manager,
            surface,
            device,
            queue,
            config,
            window: window_arc,
            render_pipeline,
            camera,
            camera_uniform,
            depth_texture,
            camera_buffer,
            camera_bind_group,
            camera_controller: camera::CameraController::new(1.0, 0.4),
            instance_buffer,
            projection,
            mouse_pressed: false,
            debug_view,
            obj_model,
            texture_manager,
            texture_bind_group,
            running: true,
            n_instances: 0,
        }
    }

    fn setup_camera(
        device: &wgpu::Device,
        config: &wgpu::SurfaceConfiguration,
    ) -> (
        camera::Camera,
        camera::Projection,
        camera::CameraUniform,
        wgpu::Buffer,
        wgpu::BindGroup,
        wgpu::BindGroupLayout,
    ) {
        let camera = camera::Camera::new((0.0, 0.0, 0.0), cgmath::Deg(0.0), cgmath::Deg(0.0));
        let projection =
            camera::Projection::new(config.width, config.height, cgmath::Deg(45.0), 0.1, 100.0);

        let mut camera_uniform = camera::CameraUniform::new();
        camera_uniform.update_view_proj(&camera, &projection);

        let camera_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Camera Buffer"),
            contents: bytemuck::cast_slice(&[camera_uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let camera_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
                label: Some("camera_bind_group_layout"),
            });

        let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &camera_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: camera_buffer.as_entire_binding(),
            }],
            label: Some("camera_bind_group"),
        });

        (
            camera,
            projection,
            camera_uniform,
            camera_buffer,
            camera_bind_group,
            camera_bind_group_layout,
        )
    }

    fn create_surface_config(
        size: PhysicalSize<u32>,
        capabilities: SurfaceCapabilities,
    ) -> wgpu::SurfaceConfiguration {
        let surface_format = capabilities
            .formats
            .iter()
            .find(|f| f.is_srgb())
            .copied()
            .unwrap_or(capabilities.formats[0]);

        wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: PresentMode::AutoNoVsync,
            alpha_mode: capabilities.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        }
    }

    fn create_device(adapter: &Adapter) -> (Device, Queue) {
        adapter
            .request_device(&wgpu::DeviceDescriptor {
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::default(),
                label: None,
                memory_hints: wgpu::MemoryHints::Performance,
                trace: wgpu::Trace::Off,
            })
            .block_on()
            .unwrap()
    }

    fn create_adapter(instance: Instance, surface: &Surface) -> Adapter {
        instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(surface),
                force_fallback_adapter: false,
            })
            .block_on()
            .unwrap()
    }

    fn create_gpu_instance() -> Instance {
        Instance::new(&wgpu::InstanceDescriptor {
            backends: wgpu::Backends::PRIMARY,
            ..Default::default()
        })
    }

    pub fn resize(&mut self, new_size: PhysicalSize<u32>) {
        self.config.width = new_size.width;
        self.config.height = new_size.height;

        self.projection.resize(new_size.width, new_size.height);

        self.surface.configure(&self.device, &self.config);

        println!("Resized to {:?} from state!", new_size);
    }

    pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let output = self.surface.get_current_texture().unwrap();
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.1,
                            g: 0.2,
                            b: 0.3,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.depth_texture.view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                occlusion_query_set: None,
                timestamp_writes: None,
            });

            if self.n_instances > 0 {
                render_pass.set_vertex_buffer(1, self.instance_buffer.slice(..));
                render_pass.set_pipeline(&self.render_pipeline);

                for mesh in &self.obj_model.meshes {
                    render_pass.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
                    render_pass
                        .set_index_buffer(mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
                    render_pass.set_bind_group(0, &self.texture_bind_group, &[]);
                    render_pass.set_bind_group(1, &self.camera_bind_group, &[]);
                    render_pass.draw_indexed(0..mesh.n_elements, 0, 0..self.n_instances as u32);
                }
            }
        }

        self.debug_view
            .render(&self.device, &self.config, &self.queue, &mut encoder, &view);

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();
        // self.window.request_redraw();

        Ok(())
    }

    fn update_instances(&mut self) -> usize {
        let instances = self.chunk_manager.gen_instances();

        debug!("{} instances to render", instances.len());

        let instance_data = instances
            .iter()
            .map(|x| x.to_raw(&self.texture_manager))
            .collect::<Vec<_>>();

        self.instance_buffer = self
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Instance Buffer"),
                contents: bytemuck::cast_slice(&instance_data),
                usage: wgpu::BufferUsages::VERTEX,
            });

        instances.len()
    }

    pub fn window(&self) -> &Window {
        &self.window
    }

    fn input(&mut self, event: &WindowEvent) {
        match event {
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        physical_key: PhysicalKey::Code(KeyCode::Escape),
                        ..
                    },
                ..
            } => self.running = false,
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        physical_key: PhysicalKey::Code(key),
                        state,
                        ..
                    },
                ..
            } => self.camera_controller.process_keyboard(*key, *state),
            WindowEvent::MouseWheel { delta, .. } => {
                self.camera_controller.process_scroll(delta);
            }
            WindowEvent::MouseInput {
                button: MouseButton::Left,
                state,
                ..
            } => {
                self.mouse_pressed = *state == ElementState::Pressed;
            }
            _ => (),
        }
    }

    pub fn update(&mut self, dt: instant::Duration) {
        self.chunk_manager.update(&self.camera, &self.projection);
        self.n_instances = self.update_instances();
        self.camera_controller.update_camera(&mut self.camera, dt);
        self.camera_uniform
            .update_view_proj(&self.camera, &self.projection);

        self.queue.write_buffer(
            &self.camera_buffer,
            0,
            bytemuck::cast_slice(&[self.camera_uniform]),
        );
    }

    fn handle_mouse_button(&mut self, button: MouseButton, pressed: bool) {
        if button == MouseButton::Left { self.mouse_pressed = pressed }
    }

    fn handle_mouse_scroll(&mut self, delta: &MouseScrollDelta) {
        self.camera_controller.process_scroll(delta);
    }
}
