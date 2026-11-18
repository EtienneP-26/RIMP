//! The egui interface drawn on top of the canvas.

use egui_wgpu::wgpu::{CommandBuffer, CommandEncoder, Device, Queue, RenderPass, TextureFormat};
use egui_wgpu::{Renderer, RendererOptions, ScreenDescriptor};
use winit::event::WindowEvent;
use winit::window::Window;

/// egui drawn on top of the canvas.
pub struct Overlay {
    ctx: egui::Context,
    state: egui_winit::State,
    renderer: Renderer,
}

/// What egui wants painted for one frame.
pub struct OverlayFrame {
    paint_jobs: Vec<egui::ClippedPrimitive>,
    freed_textures: Vec<egui::TextureId>,
    buffers: Vec<CommandBuffer>,
    screen: ScreenDescriptor,
}

impl Overlay {
    /// Prepares egui to be drawn into a window.
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

    /// Gives a window event to egui.
    pub fn on_event(&mut self, window: &Window, event: &WindowEvent) -> egui_winit::EventResponse {
        self.state.on_window_event(window, event)
    }

    /// Builds the interface and uploads what the GPU needs to draw it.
    pub fn prepare(
        &mut self,
        window: &Window,
        device: &Device,
        queue: &Queue,
        encoder: &mut CommandEncoder,
        show: impl FnMut(&egui::Context),
    ) -> OverlayFrame {
        let output = self.run_ui(window, show);
        let paint_jobs = self.ctx.tessellate(output.shapes, output.pixels_per_point);
        self.upload_textures(device, queue, &output.textures_delta);
        let screen = screen_descriptor(window, output.pixels_per_point);
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

    /// Draws the interface into a render pass, over what is already there.
    pub fn paint(&self, pass: &mut RenderPass<'static>, frame: &OverlayFrame) {
        self.renderer.render(pass, &frame.paint_jobs, &frame.screen);
    }

    /// Ends the frame: frees the unused textures.
    pub fn finish(&mut self, frame: OverlayFrame) -> Vec<CommandBuffer> {
        for id in &frame.freed_textures {
            self.renderer.free_texture(id);
        }
        frame.buffers
    }

    /// Runs the interface closure and hands the platform output to winit.
    fn run_ui(
        &mut self,
        window: &Window,
        mut show: impl FnMut(&egui::Context),
    ) -> egui::FullOutput {
        let input = self.state.take_egui_input(window);
        let mut output = self.ctx.run_ui(input, |ui| show(ui.ctx()));
        self.state
            .handle_platform_output(window, std::mem::take(&mut output.platform_output));
        output
    }

    /// Uploads the new or changed egui textures to the GPU.
    fn upload_textures(&mut self, device: &Device, queue: &Queue, delta: &egui::TexturesDelta) {
        for (id, delta) in &delta.set {
            self.renderer.update_texture(device, queue, *id, delta);
        }
    }
}

/// Builds the screen description from the window size.
fn screen_descriptor(window: &Window, pixels_per_point: f32) -> ScreenDescriptor {
    let size = window.inner_size();
    ScreenDescriptor {
        size_in_pixels: [size.width, size.height],
        pixels_per_point,
    }
}
