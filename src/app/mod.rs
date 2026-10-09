mod canvas;
mod drawing;
mod gpu;

use std::error::Error;
use std::sync::Arc;

use winit::application::ApplicationHandler;
use winit::event::{ElementState, MouseButton, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::window::{Window, WindowId};

use drawing::{Drawing, screen_to_image};
use gpu::Gpu;

/// Dark grey behind the canvas.
const BACKGROUND: wgpu::Color = wgpu::Color {
    r: 0.1,
    g: 0.1,
    b: 0.12,
    a: 1.0,
};

/// Size of the image the user paints on, in pixels.
const IMAGE_SIZE: (usize, usize) = (1024, 768);

/// Pressure used with a mouse, which does not report any.
const NO_PRESSURE: f32 = f32::NAN;

/// # The application state: the window, the GPU and the drawing.
struct App {
    window: Option<Arc<Window>>,
    gpu: Option<Gpu>,
    drawing: Drawing,
    cursor: (f32, f32),
}

impl App {
    fn new() -> Self {
        Self {
            window: None,
            gpu: None,
            drawing: Drawing::new(IMAGE_SIZE.0, IMAGE_SIZE.1),
            cursor: (0.0, 0.0),
        }
    }

    /// # Sends the drawing to the GPU and asks for a new frame.
    fn refresh(&self) {
        if let (Some(gpu), Some(window)) = (&self.gpu, &self.window) {
            gpu.update_canvas(&self.drawing.to_rgba8());
            window.request_redraw();
        }
    }

    /// # Converts a cursor position in the window into a position on the image.
    fn to_image(&self, x: f64, y: f64) -> (f32, f32) {
        let screen = self.window.as_ref().map_or((1.0, 1.0), |window| {
            let size = window.inner_size();
            (size.width as f32, size.height as f32)
        });
        let image = (IMAGE_SIZE.0 as f32, IMAGE_SIZE.1 as f32);

        screen_to_image((x as f32, y as f32), screen, image)
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }

        match create_window_and_gpu(event_loop, &self.drawing) {
            Ok((window, gpu)) => {
                self.window = Some(window);
                self.gpu = Some(gpu);
            }
            Err(error) => {
                eprintln!("rimp: cannot start the window: {error}");
                event_loop.exit();
            }
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(size) => {
                if let (Some(gpu), Some(window)) = (&mut self.gpu, &self.window) {
                    gpu.resize(size.width, size.height);
                    window.request_redraw();
                }
            }
            WindowEvent::CursorMoved { position, .. } => {
                self.cursor = self.to_image(position.x, position.y);
                if self.drawing.drag(self.cursor, NO_PRESSURE) {
                    self.refresh();
                }
            }
            WindowEvent::MouseInput {
                state,
                button: MouseButton::Left,
                ..
            } => {
                match state {
                    ElementState::Pressed => self.drawing.press(self.cursor, NO_PRESSURE),
                    ElementState::Released => self.drawing.release(),
                }
                self.refresh();
            }
            WindowEvent::RedrawRequested => {
                if let Some(gpu) = &self.gpu {
                    gpu.render(BACKGROUND);
                }
            }
            _ => {}
        }
    }
}

/// # Creates the window and connects the GPU to it.
fn create_window_and_gpu(
    event_loop: &ActiveEventLoop,
    drawing: &Drawing,
) -> Result<(Arc<Window>, Gpu), Box<dyn Error>> {
    let window =
        Arc::new(event_loop.create_window(Window::default_attributes().with_title("RIMP"))?);
    let (width, height) = drawing.size();
    let gpu = Gpu::new(
        window.clone(),
        &drawing.to_rgba8(),
        width as u32,
        height as u32,
    )?;

    Ok((window, gpu))
}

/// # Opens the RIMP window and runs until it is closed.
///
/// ## Errors
/// If the event loop cannot be created or fails while running.
pub fn run() -> Result<(), Box<dyn Error>> {
    let event_loop = EventLoop::new()?;
    event_loop.set_control_flow(ControlFlow::Wait);

    event_loop.run_app(&mut App::new())?;
    Ok(())
}
