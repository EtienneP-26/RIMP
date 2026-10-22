use egui_wgpu::wgpu::{CommandBuffer, CommandEncoder, Device, Queue, RenderPass, TextureFormat};
use egui_wgpu::{Renderer, RendererOptions, ScreenDescriptor};
use winit::event::WindowEvent;
use winit::window::Window;

/// # egui drawn on top of the canvas.
///
/// Wraps the three egui pieces: the context (the interface logic), the link with winit
/// (mouse and keyboard) and the wgpu renderer.
pub struct Overlay {
    ctx: egui::Context,
    state: egui_winit::State,
    renderer: Renderer,
}

/// # What egui wants painted for one frame.
pub struct OverlayFrame {
    paint_jobs: Vec<egui::ClippedPrimitive>,
    freed_textures: Vec<egui::TextureId>,
    buffers: Vec<CommandBuffer>,
    screen: ScreenDescriptor,
}

impl Overlay {
    /// # Prepares egui to be drawn into a window.
    ///
    /// ## Arguments
    /// * `window` - The window egui receives its input from
    /// * `device` - The GPU device
    /// * `format` - The format of the surface that will be drawn on
    pub fn new(window: &Window, device: &Device, format: TextureFormat) -> Self {
        let ctx = egui::Context::default();
        let state = egui_winit::State::new(
            ctx.clone(),
            egui::ViewportId::ROOT,
            window,
            Some(window.scale_factor() as f32),
            None,
            None,
        );
        let renderer = Renderer::new(device, format, RendererOptions::default());

        Self {
            ctx,
            state,
            renderer,
        }
    }

    /// # Gives a window event to egui.
    ///
    /// ## Returns
    /// What egui did with it: `consumed` (the event is for the interface, not the canvas)
    /// and `repaint` (a new frame is needed).
    pub fn on_event(&mut self, window: &Window, event: &WindowEvent) -> egui_winit::EventResponse {
        self.state.on_window_event(window, event)
    }

    /// # Builds the interface and uploads what the GPU needs to draw it.
    ///
    /// ## Arguments
    /// * `window` - The window the interface is drawn in
    /// * `device` - The GPU device
    /// * `queue` - The GPU queue
    /// * `encoder` - The encoder of the current frame
    /// * `show` - Describes the interface (windows, labels...)
    ///
    /// ## Returns
    /// The frame to give to `paint`, then to `finish`.
    pub fn prepare(
        &mut self,
        window: &Window,
        device: &Device,
        queue: &Queue,
        encoder: &mut CommandEncoder,
        mut show: impl FnMut(&egui::Context),
    ) -> OverlayFrame {
        let input = self.state.take_egui_input(window);
        let output = self.ctx.run_ui(input, |ui| show(ui.ctx()));
        self.state
            .handle_platform_output(window, output.platform_output);

        let paint_jobs = self.ctx.tessellate(output.shapes, output.pixels_per_point);
        for (id, delta) in &output.textures_delta.set {
            self.renderer.update_texture(device, queue, *id, delta);
        }

        let size = window.inner_size();
        let screen = ScreenDescriptor {
            size_in_pixels: [size.width, size.height],
            pixels_per_point: output.pixels_per_point,
        };
        let buffers = self
            .renderer
            .update_buffers(device, queue, encoder, &paint_jobs, &screen);

        OverlayFrame {
            paint_jobs,
            freed_textures: output.textures_delta.free,
            buffers,
            screen,
        }
    }

    /// # Draws the interface into a render pass, over what is already there.
    pub fn paint(&self, pass: &mut RenderPass<'static>, frame: &OverlayFrame) {
        self.renderer
            .render(pass, &frame.paint_jobs, &frame.screen);
    }

    /// # Ends the frame: frees the unused textures.
    ///
    /// ## Returns
    /// The command buffers to submit to the GPU before the frame's own commands.
    pub fn finish(&mut self, frame: OverlayFrame) -> Vec<CommandBuffer> {
        for id in &frame.freed_textures {
            self.renderer.free_texture(id);
        }
        frame.buffers
    }
}
