use glyphon::{
    Attrs, Buffer, Cache, Color, Family, FontSystem, Metrics, Resolution, Shaping, SwashCache,
    TextArea, TextAtlas, TextBounds, TextRenderer, Viewport,
};

// Controls the things to be rendered for the debug view
pub struct DebugView {
    font_system: FontSystem,
    viewport: Viewport,
    atlas: TextAtlas,
    renderer: TextRenderer,
    buffer: Buffer,
    swash_cache: SwashCache,
    pub view_active: bool,
}

impl DebugView {
    pub fn new(
        device: &wgpu::Device,
        config: &wgpu::SurfaceConfiguration,
        queue: &wgpu::Queue,
        scale_factor: f64,
    ) -> Self {
        let physical_width = (config.width as f64 * scale_factor) as f32;
        let physical_height = (config.height as f64 * scale_factor) as f32;

        let mut font_system = FontSystem::new();
        let swash_cache = SwashCache::new();
        let cache = Cache::new(device);
        let viewport = Viewport::new(device, &cache);
        let mut atlas = TextAtlas::new(device, queue, &cache, wgpu::TextureFormat::Bgra8UnormSrgb);
        let text_renderer =
            TextRenderer::new(&mut atlas, device, wgpu::MultisampleState::default(), None);
        let mut text_buffer = Buffer::new(&mut font_system, Metrics::new(30.0, 42.0));

        text_buffer.set_size(
            &mut font_system,
            Some(physical_width),
            Some(physical_height),
        );
        text_buffer.set_text(
            &mut font_system,
            "Debug Information\n",
            &Attrs::new().family(Family::SansSerif),
            Shaping::Advanced,
        );
        text_buffer.shape_until_scroll(&mut font_system, false);

        Self {
            font_system,
            viewport,
            atlas,
            renderer: text_renderer,
            buffer: text_buffer,
            swash_cache,
            view_active: true,
        }
    }

    pub fn update_text(&mut self, txt: &str) {
        self.buffer.set_text(
            &mut self.font_system,
            txt,
            &Attrs::new().family(Family::SansSerif),
            Shaping::Advanced,
        );
    }

    pub fn render(
        &mut self,
        device: &wgpu::Device,
        config: &wgpu::SurfaceConfiguration,
        queue: &wgpu::Queue,
        encoder: &mut wgpu::CommandEncoder,
        view: &wgpu::TextureView,
    ) {
        if !self.view_active {
            return;
        }

        self.viewport.update(
            queue,
            Resolution {
                width: config.width,
                height: config.height,
            },
        );

        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("text_render_pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        let _ = self.renderer.prepare(
            device,
            queue,
            &mut self.font_system,
            &mut self.atlas,
            &self.viewport,
            [TextArea {
                buffer: &self.buffer,
                left: 10.0,
                top: 10.0,
                scale: 1.0,
                bounds: TextBounds {
                    left: 0,
                    top: 0,
                    right: 600,
                    bottom: 600,
                },
                default_color: Color::rgb(255, 255, 255),
                custom_glyphs: &[],
            }],
            &mut self.swash_cache,
        );

        let _ = self.renderer.render(&self.atlas, &self.viewport, &mut pass);

        self.atlas.trim();
    }
}
