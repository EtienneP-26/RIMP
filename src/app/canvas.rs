use wgpu::{
    AddressMode, BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayoutDescriptor,
    BindGroupLayoutEntry, BindingResource, BindingType, ColorTargetState, Device, Extent3d,
    FilterMode, FragmentState, MultisampleState, Origin3d, PipelineLayoutDescriptor,
    PrimitiveState, Queue, RenderPass, RenderPipeline, RenderPipelineDescriptor,
    SamplerBindingType, SamplerDescriptor, ShaderModuleDescriptor, ShaderSource, ShaderStages,
    TexelCopyBufferLayout, TexelCopyTextureInfo, Texture, TextureAspect, TextureDescriptor,
    TextureDimension, TextureFormat, TextureSampleType, TextureUsages, TextureViewDescriptor,
    TextureViewDimension, VertexState,
};

/// # An RGBA8 image drawn over the whole window.
pub struct Canvas {
    pipeline: RenderPipeline,
    bind_group: BindGroup,
    texture: Texture,
    size: Extent3d,
}

impl Canvas {
    /// # Uploads an image to the GPU and prepares to draw it full screen.
    ///
    /// ## Arguments
    /// * `device` - The GPU device
    /// * `queue` - The GPU queue, used to upload the pixels
    /// * `target_format` - The format of the surface that will be drawn on
    /// * `rgba` - The pixels, 4 bytes each, row by row
    /// * `width` - Image width in pixels
    /// * `height` - Image height in pixels
    ///
    /// ## Returns
    /// A `Canvas` ready to be drawn with `draw`.
    pub fn new(
        device: &Device,
        queue: &Queue,
        target_format: TextureFormat,
        rgba: &[u8],
        width: u32,
        height: u32,
    ) -> Self {
        let size = Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        };
        let texture = create_texture(device, size);
        write_pixels(queue, &texture, size, rgba);
        let view = texture.create_view(&TextureViewDescriptor::default());
        let sampler = device.create_sampler(&SamplerDescriptor {
            address_mode_u: AddressMode::ClampToEdge,
            address_mode_v: AddressMode::ClampToEdge,
            mag_filter: FilterMode::Nearest,
            min_filter: FilterMode::Nearest,
            ..Default::default()
        });

        let layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("canvas layout"),
            entries: &[
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Texture {
                        sample_type: TextureSampleType::Float { filterable: true },
                        view_dimension: TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Sampler(SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });

        let bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: Some("canvas bind group"),
            layout: &layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: BindingResource::TextureView(&view),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: BindingResource::Sampler(&sampler),
                },
            ],
        });

        Self {
            pipeline: create_pipeline(device, &layout, target_format),
            bind_group,
            texture,
            size,
        }
    }

    /// # Replaces the pixels of the image already on the GPU.
    ///
    /// ## Arguments
    /// * `queue` - The GPU queue, used to upload the pixels
    /// * `rgba` - The new pixels, same size as the image given to `new`
    pub fn update(&self, queue: &Queue, rgba: &[u8]) {
        write_pixels(queue, &self.texture, self.size, rgba);
    }

    /// # Draws the image over the whole render target.
    ///
    /// ## Arguments
    /// * `pass` - The render pass to draw into
    pub fn draw(&self, pass: &mut RenderPass<'_>) {
        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, &self.bind_group, &[]);
        pass.draw(0..3, 0..1);
    }
}

/// # Creates an empty GPU texture of the given size.
fn create_texture(device: &Device, size: Extent3d) -> Texture {
    device.create_texture(&TextureDescriptor {
        label: Some("canvas texture"),
        size,
        mip_level_count: 1,
        sample_count: 1,
        dimension: TextureDimension::D2,
        format: TextureFormat::Rgba8UnormSrgb,
        usage: TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
        view_formats: &[],
    })
}

/// # Copies RGBA8 pixels into a texture.
fn write_pixels(queue: &Queue, texture: &Texture, size: Extent3d, rgba: &[u8]) {
    queue.write_texture(
        TexelCopyTextureInfo {
            texture,
            mip_level: 0,
            origin: Origin3d::ZERO,
            aspect: TextureAspect::All,
        },
        rgba,
        TexelCopyBufferLayout {
            offset: 0,
            bytes_per_row: Some(4 * size.width),
            rows_per_image: Some(size.height),
        },
        size,
    );
}

/// # Creates the pipeline that draws a texture over the whole screen.
fn create_pipeline(
    device: &Device,
    bind_group_layout: &wgpu::BindGroupLayout,
    target_format: TextureFormat,
) -> RenderPipeline {
    let shader = device.create_shader_module(ShaderModuleDescriptor {
        label: Some("canvas shader"),
        source: ShaderSource::Wgsl(include_str!("canvas.wgsl").into()),
    });
    let layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
        label: Some("canvas pipeline layout"),
        bind_group_layouts: &[Some(bind_group_layout)],
        ..Default::default()
    });

    device.create_render_pipeline(&RenderPipelineDescriptor {
        label: Some("canvas pipeline"),
        layout: Some(&layout),
        vertex: VertexState {
            module: &shader,
            entry_point: Some("vs_main"),
            compilation_options: Default::default(),
            buffers: &[],
        },
        fragment: Some(FragmentState {
            module: &shader,
            entry_point: Some("fs_main"),
            compilation_options: Default::default(),
            targets: &[Some(ColorTargetState::from(target_format))],
        }),
        primitive: PrimitiveState::default(),
        depth_stencil: None,
        multisample: MultisampleState::default(),
        multiview_mask: None,
        cache: None,
    })
}
