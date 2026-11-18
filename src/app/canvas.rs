//! A full-window RGBA8 image drawn on the GPU.

use egui_wgpu::wgpu;
use glam::{Mat3, Vec2};
use wgpu::util::{BufferInitDescriptor, DeviceExt};
use wgpu::{
    AddressMode, BindGroup, BindGroupDescriptor, BindGroupEntry, BindingResource, Buffer,
    BufferUsages, ColorTargetState, Device, Extent3d, FilterMode, FragmentState, MultisampleState,
    Origin3d, PrimitiveState, Queue, RenderPass, RenderPipeline, RenderPipelineDescriptor, Sampler,
    SamplerDescriptor, ShaderModule, ShaderModuleDescriptor, ShaderSource, TexelCopyBufferLayout,
    TexelCopyTextureInfo, Texture, TextureAspect, TextureDescriptor, TextureDimension,
    TextureFormat, TextureUsages, TextureViewDescriptor, VertexState,
};

use crate::core::viewport::Viewport;

/// An RGBA8 image drawn over the whole window.
pub struct Canvas {
    pipeline: RenderPipeline,
    bind_group: BindGroup,
    texture: Texture,
    size: Extent3d,
    view_buffer: Buffer,
}

impl Canvas {
    /// Uploads an image to the GPU and prepares to draw it full screen.
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
        let pipeline = create_pipeline(device, target_format);
        let view_buffer = create_view_buffer(device, width, height);
        let bind_group = create_bind_group(device, &pipeline, &texture, &view_buffer);
        Self {
            pipeline,
            bind_group,
            texture,
            size,
            view_buffer,
        }
    }

    /// Replaces one clipped `(x, y, w, h)` rectangle of the image on the GPU.
    pub fn update_region(&self, queue: &Queue, rect: (u32, u32, u32, u32), rgba: &[u8]) {
        let (x, y, w, h) = rect;
        let w = w.min(self.size.width.saturating_sub(x));
        let h = h.min(self.size.height.saturating_sub(y));
        if w == 0 || h == 0 || rgba.len() < (4 * w * h) as usize {
            return;
        }
        let origin = Origin3d { x, y, z: 0 };
        let extent = Extent3d {
            width: w,
            height: h,
            depth_or_array_layers: 1,
        };
        write_region(queue, &self.texture, origin, extent, rgba);
    }

    /// Sends the viewport to the shader.
    pub fn set_viewport(&self, queue: &Queue, viewport: &Viewport) {
        let bytes = view_bytes(viewport, self.size.width, self.size.height);
        queue.write_buffer(&self.view_buffer, 0, &bytes);
    }

    /// Draws the image over the whole render target.
    pub fn draw(&self, pass: &mut RenderPass<'_>) {
        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, &self.bind_group, &[]);
        pass.draw(0..3, 0..1);
    }
}

/// Builds the matrix taking a screen uv to a canvas uv.
fn uv_matrix(viewport: &Viewport, width: u32, height: u32) -> Mat3 {
    let size = Vec2::new(width.max(1) as f32, height.max(1) as f32);
    Mat3::from_diagonal(size.recip().extend(1.0))
        * viewport.matrix().inverse()
        * Mat3::from_diagonal(size.extend(1.0))
}

/// Builds the uniform bytes: a mat3x3 as three columns padded to 16 bytes.
fn view_bytes(viewport: &Viewport, width: u32, height: u32) -> [u8; 48] {
    let m = uv_matrix(viewport, width, height);
    let mut bytes = [0u8; 48];
    for (i, col) in [m.x_axis, m.y_axis, m.z_axis].iter().enumerate() {
        for (j, v) in col.to_array().iter().enumerate() {
            bytes[i * 16 + j * 4..][..4].copy_from_slice(&v.to_ne_bytes());
        }
    }
    bytes
}

/// Creates an empty GPU texture of the given size.
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

/// Creates the nearest-neighbour sampler that clamps at the edges.
fn create_sampler(device: &Device) -> Sampler {
    device.create_sampler(&SamplerDescriptor {
        address_mode_u: AddressMode::ClampToEdge,
        address_mode_v: AddressMode::ClampToEdge,
        mag_filter: FilterMode::Nearest,
        min_filter: FilterMode::Nearest,
        ..Default::default()
    })
}

/// Creates the uniform buffer holding the default viewport.
fn create_view_buffer(device: &Device, width: u32, height: u32) -> Buffer {
    device.create_buffer_init(&BufferInitDescriptor {
        label: Some("canvas view"),
        contents: &view_bytes(&Viewport::default(), width, height),
        usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
    })
}

/// Binds the texture, sampler and view buffer to the pipeline.
fn create_bind_group(
    device: &Device,
    pipeline: &RenderPipeline,
    texture: &Texture,
    view_buffer: &Buffer,
) -> BindGroup {
    let view = texture.create_view(&TextureViewDescriptor::default());
    let sampler = create_sampler(device);
    device.create_bind_group(&BindGroupDescriptor {
        label: Some("canvas bind group"),
        layout: &pipeline.get_bind_group_layout(0),
        entries: &[
            bind_entry(0, BindingResource::TextureView(&view)),
            bind_entry(1, BindingResource::Sampler(&sampler)),
            bind_entry(2, view_buffer.as_entire_binding()),
        ],
    })
}

/// Makes one bind group entry.
fn bind_entry(binding: u32, resource: BindingResource<'_>) -> BindGroupEntry<'_> {
    BindGroupEntry { binding, resource }
}

/// Copies RGBA8 pixels into a whole texture.
fn write_pixels(queue: &Queue, texture: &Texture, size: Extent3d, rgba: &[u8]) {
    write_region(queue, texture, Origin3d::ZERO, size, rgba);
}

/// Copies RGBA8 pixels into a rectangle of a texture.
fn write_region(queue: &Queue, texture: &Texture, origin: Origin3d, size: Extent3d, rgba: &[u8]) {
    let layout = TexelCopyBufferLayout {
        offset: 0,
        bytes_per_row: Some(4 * size.width),
        rows_per_image: Some(size.height),
    };
    queue.write_texture(texture_target(texture, origin), rgba, layout, size);
}

/// Describes the destination of a texture copy.
fn texture_target(texture: &Texture, origin: Origin3d) -> TexelCopyTextureInfo<'_> {
    TexelCopyTextureInfo {
        texture,
        mip_level: 0,
        origin,
        aspect: TextureAspect::All,
    }
}

/// Compiles the canvas shader.
fn create_shader(device: &Device) -> ShaderModule {
    device.create_shader_module(ShaderModuleDescriptor {
        label: Some("canvas shader"),
        source: ShaderSource::Wgsl(include_str!("canvas.wgsl").into()),
    })
}

/// Creates the pipeline that draws a texture over the whole screen.
fn create_pipeline(device: &Device, target_format: TextureFormat) -> RenderPipeline {
    let shader = create_shader(device);
    let targets = [Some(ColorTargetState::from(target_format))];
    device.create_render_pipeline(&RenderPipelineDescriptor {
        label: Some("canvas pipeline"),
        layout: None,
        vertex: VertexState {
            module: &shader,
            entry_point: Some("vs_main"),
            compilation_options: Default::default(),
            buffers: &[],
        },
        fragment: Some(fragment_state(&shader, &targets)),
        primitive: PrimitiveState::default(),
        depth_stencil: None,
        multisample: MultisampleState::default(),
        multiview_mask: None,
        cache: None,
    })
}

/// Describes the fragment stage writing to the given targets.
fn fragment_state<'a>(
    shader: &'a ShaderModule,
    targets: &'a [Option<ColorTargetState>],
) -> FragmentState<'a> {
    FragmentState {
        module: shader,
        entry_point: Some("fs_main"),
        compilation_options: Default::default(),
        targets,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// An identity viewport leaves the uv unchanged.
    #[test]
    fn identity_viewport_keeps_uv() {
        let m = uv_matrix(&Viewport::default(), 1024, 768);
        let uv = m.transform_point2(Vec2::new(0.25, 0.75));
        assert!((uv - Vec2::new(0.25, 0.75)).length() < 1e-5);
    }

    /// Zooming maps the uv around the offset.
    #[test]
    fn zoom_maps_uv_around_offset() {
        let v = Viewport {
            zoom: 2.0,
            ..Default::default()
        };
        let uv = uv_matrix(&v, 100, 100).transform_point2(Vec2::new(0.5, 0.5));
        assert!((uv - Vec2::new(0.25, 0.25)).length() < 1e-5);
    }
}
