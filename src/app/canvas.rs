use egui_wgpu::wgpu;
use glam::{Mat3, Vec2};
use wgpu::util::{BufferInitDescriptor, DeviceExt};
use wgpu::{
    AddressMode, Buffer, BufferUsages, BindGroup, BindGroupDescriptor, BindGroupEntry, BindingResource,
    ColorTargetState, Device, Extent3d, FilterMode, FragmentState, MultisampleState, Origin3d,
    PrimitiveState, Queue, RenderPass, RenderPipeline, RenderPipelineDescriptor,
    SamplerDescriptor, ShaderModuleDescriptor, ShaderSource, TexelCopyBufferLayout,
    TexelCopyTextureInfo, Texture, TextureAspect, TextureDescriptor, TextureDimension,
    TextureFormat, TextureUsages, TextureViewDescriptor, VertexState,
};

use crate::core::viewport::Viewport;

/// # An RGBA8 image drawn over the whole window.
pub struct Canvas {
    pipeline: RenderPipeline,
    bind_group: BindGroup,
    texture: Texture,
    size: Extent3d,
    view_buffer: Buffer,
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

        let pipeline = create_pipeline(device, target_format);

        let view_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("canvas view"),
            contents: &view_bytes(&Viewport::default(), width, height),
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        });

        let bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: Some("canvas bind group"),
            layout: &pipeline.get_bind_group_layout(0),
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: BindingResource::TextureView(&view),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: BindingResource::Sampler(&sampler),
                },
                BindGroupEntry {
                    binding: 2,
                    resource: view_buffer.as_entire_binding(),
                },
            ],
        });

        Self {
            pipeline,
            bind_group,
            texture,
            size,
            view_buffer,
        }
    }

    /// # Replaces one rectangle of the image already on the GPU.
    ///
    /// ## Arguments
    /// * `queue` - The GPU queue, used to upload the pixels
    /// * `rect` - `(x, y, width, height)` in pixels; clipped to the image
    /// * `rgba` - The rectangle's pixels, 4 bytes each, row by row
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

    /// # Sends the viewport to the shader.
    ///
    /// ## Arguments
    /// * `queue` - The GPU queue
    /// * `viewport` - Canvas -> screen transform, in image pixels
    pub fn set_viewport(&self, queue: &Queue, viewport: &Viewport) {
        let bytes = view_bytes(viewport, self.size.width, self.size.height);
        queue.write_buffer(&self.view_buffer, 0, &bytes);
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

/// # Matrix taking a screen uv to a canvas uv, for an image of `width` x `height`.
///
/// The screen uv is scaled to image pixels (the image fills the window), the inverse
/// viewport is applied, and the result is scaled back to a uv.
fn uv_matrix(viewport: &Viewport, width: u32, height: u32) -> Mat3 {
    let size = Vec2::new(width.max(1) as f32, height.max(1) as f32);
    Mat3::from_diagonal(size.recip().extend(1.0))
        * viewport.matrix().inverse()
        * Mat3::from_diagonal(size.extend(1.0))
}

/// # The uniform bytes: a mat3x3 as three columns padded to 16 bytes.
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
    write_region(queue, texture, Origin3d::ZERO, size, rgba);
}

/// # Copies RGBA8 pixels into a rectangle of a texture.
fn write_region(
    queue: &Queue,
    texture: &Texture,
    origin: Origin3d,
    size: Extent3d,
    rgba: &[u8],
) {
    queue.write_texture(
        TexelCopyTextureInfo {
            texture,
            mip_level: 0,
            origin,
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
///
/// The layout of the bindings is worked out by wgpu from the shader.
fn create_pipeline(device: &Device, target_format: TextureFormat) -> RenderPipeline {
    let shader = device.create_shader_module(ShaderModuleDescriptor {
        label: Some("canvas shader"),
        source: ShaderSource::Wgsl(include_str!("canvas.wgsl").into()),
    });

    device.create_render_pipeline(&RenderPipelineDescriptor {
        label: Some("canvas pipeline"),
        layout: None,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identity_viewport_keeps_uv() {
        let m = uv_matrix(&Viewport::default(), 1024, 768);
        let uv = m.transform_point2(Vec2::new(0.25, 0.75));
        assert!((uv - Vec2::new(0.25, 0.75)).length() < 1e-5);
    }

    #[test]
    fn zoom_maps_uv_around_offset() {
        let v = Viewport { zoom: 2.0, ..Default::default() };
        let uv = uv_matrix(&v, 100, 100).transform_point2(Vec2::new(0.5, 0.5));
        assert!((uv - Vec2::new(0.25, 0.25)).length() < 1e-5);
    }
}
