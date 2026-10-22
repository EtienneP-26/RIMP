use egui_wgpu::wgpu;
use std::error::Error;
use std::sync::Arc;

use wgpu::{
    Color, CurrentSurfaceTexture, Device, DeviceDescriptor, Instance, InstanceDescriptor, LoadOp,
    Operations, Queue, RenderPassColorAttachment, RenderPassDescriptor, RequestAdapterOptions,
    StoreOp, Surface, SurfaceConfiguration, TextureViewDescriptor,
};
use winit::event::WindowEvent;
use winit::window::Window;

use super::canvas::Canvas;
use super::overlay::Overlay;

/// # The GPU objects needed to draw into one window.
pub struct Gpu {
    surface: Surface<'static>,
    device: Device,
    queue: Queue,
    config: SurfaceConfiguration,
    canvas: Canvas,
    overlay: Overlay,
}

impl Gpu {
    /// # Connects to the GPU and prepares the window to be drawn on.
    ///
    /// ## Arguments
    /// * `window` - The window to draw into
    /// * `rgba` - The image shown over the whole window, 4 bytes per pixel
    /// * `width` - Image width in pixels
    /// * `height` - Image height in pixels
    ///
    /// ## Returns
    /// The ready-to-use `Gpu`.
    ///
    /// ## Errors
    /// If no graphics adapter is found, or if the surface or device cannot be created.
    pub fn new(
        window: Arc<Window>,
        rgba: &[u8],
        width: u32,
        height: u32,
    ) -> Result<Self, Box<dyn Error>> {
        let size = window.inner_size();
        let instance = Instance::new(InstanceDescriptor::new_without_display_handle());
        let surface = instance.create_surface(window.clone())?;

        let adapter = pollster::block_on(instance.request_adapter(&RequestAdapterOptions {
            compatible_surface: Some(&surface),
            ..Default::default()
        }))?;
        let (device, queue) =
            pollster::block_on(adapter.request_device(&DeviceDescriptor::default()))?;

        let config = surface
            .get_default_config(&adapter, size.width.max(1), size.height.max(1))
            .ok_or("the surface is not supported by this graphics adapter")?;
        surface.configure(&device, &config);
        let canvas = Canvas::new(&device, &queue, config.format, rgba, width, height);
        let overlay = Overlay::new(&window, &device, config.format);

        Ok(Self {
            surface,
            device,
            queue,
            config,
            canvas,
            overlay,
        })
    }

    /// # Adapts the drawing surface to a new window size.
    ///
    /// A size of zero (minimized window) is ignored.
    ///
    /// ## Arguments
    /// * `width` - New width in pixels
    /// * `height` - New height in pixels
    pub fn resize(&mut self, width: u32, height: u32) {
        if width == 0 || height == 0 {
            return;
        }

        self.config.width = width;
        self.config.height = height;
        self.surface.configure(&self.device, &self.config);
    }

    /// # Sends new pixels to the image shown in the window.
    ///
    /// ## Arguments
    /// * `rgba` - The pixels, same size as the image given to `new`
    pub fn update_canvas(&self, rgba: &[u8]) {
        self.canvas.update(&self.queue, rgba);
    }

    /// # Gives a window event to the interface.
    ///
    /// ## Returns
    /// What the interface did with it, see `Overlay::on_event`.
    pub fn on_window_event(
        &mut self,
        window: &Window,
        event: &WindowEvent,
    ) -> egui_winit::EventResponse {
        self.overlay.on_event(window, event)
    }

    /// # Draws one frame: the background colour, the image over the whole window, then the interface.
    ///
    /// A frame that cannot be acquired (window hidden, surface outdated) is skipped.
    ///
    /// ## Arguments
    /// * `color` - The background colour
    /// * `window` - The window being drawn
    /// * `show` - Describes the interface drawn over the image
    pub fn render(&mut self, color: Color, window: &Window, show: impl FnMut(&egui::Context)) {
        let frame = match self.surface.get_current_texture() {
            CurrentSurfaceTexture::Success(frame) | CurrentSurfaceTexture::Suboptimal(frame) => {
                frame
            }
            CurrentSurfaceTexture::Outdated => {
                self.surface.configure(&self.device, &self.config);
                return;
            }
            _ => return,
        };

        let view = frame.texture.create_view(&TextureViewDescriptor::default());
        let mut encoder = self.device.create_command_encoder(&Default::default());
        let overlay_frame =
            self.overlay
                .prepare(window, &self.device, &self.queue, &mut encoder, show);

        let mut pass = encoder.begin_render_pass(&RenderPassDescriptor {
            color_attachments: &[Some(RenderPassColorAttachment {
                view: &view,
                depth_slice: None,
                resolve_target: None,
                ops: Operations {
                    load: LoadOp::Clear(color),
                    store: StoreOp::Store,
                },
            })],
            ..Default::default()
        });
        self.canvas.draw(&mut pass);
        self.overlay.paint(&mut pass.forget_lifetime(), &overlay_frame);

        let mut commands = self.overlay.finish(overlay_frame);
        commands.push(encoder.finish());
        self.queue.submit(commands);
        frame.present();
    }
}
