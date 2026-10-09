mod canvas;
mod gpu;

use std::error::Error;
use std::sync::Arc;

use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::window::{Window, WindowId};

use crate::core::{Pixel, TILE_SIZE, Tile};
use crate::io::tile_to_rgba8;
use gpu::Gpu;

/// Dark grey behind the canvas.
const BACKGROUND: wgpu::Color = wgpu::Color {
    r: 0.1,
    g: 0.1,
    b: 0.12,
    a: 1.0,
};

/// # The application state: the window and the GPU, created once the loop starts.
#[derive(Default)]
struct App {
    window: Option<Arc<Window>>,
    gpu: Option<Gpu>,
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }

        match create_window_and_gpu(event_loop) {
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
) -> Result<(Arc<Window>, Gpu), Box<dyn Error>> {
    let window =
        Arc::new(event_loop.create_window(Window::default_attributes().with_title("RIMP"))?);
    let rgba = tile_to_rgba8(&checkerboard());
    let gpu = Gpu::new(window.clone(), &rgba, TILE_SIZE as u32, TILE_SIZE as u32)?;

    Ok((window, gpu))
}

/// # Builds a test tile: a light and dark grey checkerboard of 8-pixel squares.
fn checkerboard() -> Tile {
    let mut tile = Tile::new();

    for y in 0..TILE_SIZE {
        for x in 0..TILE_SIZE {
            let level = if (x / 8 + y / 8) % 2 == 0 { 0.8 } else { 0.3 };
            tile.set(
                x,
                y,
                Pixel {
                    r: level,
                    g: level,
                    b: level,
                    a: 1.0,
                },
            );
        }
    }

    tile
}

/// # Opens the RIMP window and runs until it is closed.
///
/// ## Errors
/// If the event loop cannot be created or fails while running.
pub fn run() -> Result<(), Box<dyn Error>> {
    let event_loop = EventLoop::new()?;
    event_loop.set_control_flow(ControlFlow::Wait);

    event_loop.run_app(&mut App::default())?;
    Ok(())
}
