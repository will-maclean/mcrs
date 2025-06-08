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
    ) -> Self {
        let mut font_system = FontSystem::new();
        let swash_cache = SwashCache::new();
        let cache = Cache::new(device);
        let viewport = Viewport::new(device, &cache);
        let mut atlas = TextAtlas::new(device, &queue, &cache, wgpu::TextureFormat::Bgra8UnormSrgb);
        let text_renderer =
            TextRenderer::new(&mut atlas, device, wgpu::MultisampleState::default(), None);
        let mut text_buffer = Buffer::new(&mut font_system, Metrics::new(30.0, 42.0));

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

    pub fn render(
        &mut self,
        device: &wgpu::Device,
        config: &wgpu::SurfaceConfiguration,
        queue: &wgpu::Queue,
        pass: &mut wgpu::RenderPass,
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

        self.renderer.prepare(
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
                    bottom: 160,
                },
                default_color: Color::rgb(255, 255, 255),
                custom_glyphs: &[],
            }],
            &mut self.swash_cache,
        );

        self.renderer.render(&self.atlas, &self.viewport, pass);

        self.atlas.trim();
    }
}
