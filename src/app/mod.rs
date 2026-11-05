mod canvas;
mod debug;
mod drawing;
mod gpu;
mod overlay;
mod pen;
mod view_input;

use std::error::Error;
use std::sync::Arc;
use std::time::{Duration, Instant};

use egui_wgpu::wgpu;
use octotablet::Manager;
use octotablet::builder::Builder;
use winit::application::ApplicationHandler;
use winit::event::{ElementState, MouseButton, WindowEvent};
use winit::keyboard::{Key, NamedKey};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::window::{Window, WindowId};

use debug::DebugStats;
use drawing::{Drawing, screen_to_image};
use gpu::Gpu;
use pen::PenEvent;
use view_input::ViewInput;

use crate::core::viewport::Viewport;

/// Dark grey behind the canvas.
const BACKGROUND: wgpu::Color = wgpu::Color {
    r: 0.1,
    g: 0.1,
    b: 0.12,
    a: 1.0,
};

/// Size of the image the user paints on, in pixels.
const IMAGE_SIZE: (usize, usize) = (1024, 768);

/// Pressure used when the device reports none (a mouse).
const FULL_PRESSURE: f32 = 1.0;

/// How often the pen events are read when nothing else wakes the loop.
const PEN_POLL: Duration = Duration::from_millis(10);

/// # The application state: the window, the GPU and the drawing.
struct App {
    window: Option<Arc<Window>>,
    gpu: Option<Gpu>,
    tablet: Option<Manager>,
    drawing: Drawing,
    viewport: Viewport,
    view_input: ViewInput,
    cursor: (f32, f32),
    pressed: bool,
    canvas_dirty: bool,
    stats: DebugStats,
}

impl App {
    fn new() -> Self {
        Self {
            window: None,
            gpu: None,
            tablet: None,
            drawing: Drawing::new(IMAGE_SIZE.0, IMAGE_SIZE.1),
            viewport: Viewport::default(),
            view_input: ViewInput::default(),
            cursor: (0.0, 0.0),
            pressed: false,
            canvas_dirty: false,
            stats: DebugStats::default(),
        }
    }

    /// # Converts a position in the window (physical pixels) into the view's screen space.
    fn to_view_space(&self, position: (f32, f32)) -> (f32, f32) {
        let screen = self.window.as_ref().map_or((1.0, 1.0), |window| {
            let size = window.inner_size();
            (size.width as f32, size.height as f32)
        });
        let image = (IMAGE_SIZE.0 as f32, IMAGE_SIZE.1 as f32);

        screen_to_image(position, screen, image)
    }

    /// # Converts a position in the window (physical pixels) into a position on the image.
    fn to_image(&self, position: (f32, f32)) -> (f32, f32) {
        pen::to_canvas(self.to_view_space(position), &self.viewport)
    }

    /// # Pans the view if a trigger is held; returns `true` if the cursor move was a pan.
    fn pan_to(&mut self, position: (f32, f32)) -> bool {
        let view_position = self.to_view_space(position).into();
        let moved = self.view_input.cursor_moved(view_position, &mut self.viewport);
        if moved {
            self.request_redraw();
        }
        self.view_input.is_panning()
    }

    /// # Updates the pan triggers (Space key, middle button).
    fn track_pan_trigger(&mut self, event: &WindowEvent) {
        match event {
            WindowEvent::KeyboardInput { event, .. }
                if event.logical_key == Key::Named(NamedKey::Space) =>
            {
                self.view_input.set_space(event.state.is_pressed());
            }
            WindowEvent::MouseInput {
                state,
                button: MouseButton::Middle,
                ..
            } => self.view_input.set_middle(state.is_pressed()),
            _ => {}
        }
    }

    /// # Paints at a point of the image and asks for a new frame.
    ///
    /// The image is only sent to the GPU when the frame is drawn, so many events
    /// between two frames cost a single upload.
    fn paint(&mut self, point: (f32, f32), pressure: f32) {
        self.drawing.paint(point, pressure);
        self.stats.record_event(pressure);
        self.canvas_dirty = true;
        self.request_redraw();
    }

    /// # Ends the stroke in progress.
    fn release(&mut self) {
        self.pressed = false;
        self.drawing.lift();
    }

    fn request_redraw(&self) {
        if let Some(window) = &self.window {
            window.request_redraw();
        }
    }

    /// # Lets the interface see an event first.
    ///
    /// ## Returns
    /// `true` if the event was meant for the interface (a click on an egui window, for
    /// example) and must not draw on the canvas. A button release always goes through, so
    /// that a stroke started on the canvas can never get stuck.
    fn interface_takes(&mut self, event: &WindowEvent) -> bool {
        let (Some(gpu), Some(window)) = (&mut self.gpu, &self.window) else {
            return false;
        };

        let response = gpu.on_window_event(window, event);
        if response.repaint {
            window.request_redraw();
        }

        let is_release = matches!(
            event,
            WindowEvent::MouseInput {
                state: ElementState::Released,
                ..
            }
        );
        response.consumed && !is_release
    }

    /// # Applies one pen event to the drawing.
    fn handle_pen(&mut self, event: PenEvent) {
        match event {
            PenEvent::Down => self.pressed = true,
            PenEvent::Up => self.release(),
            PenEvent::Move { position, pressure } if self.pressed => {
                let scale = self.window.as_ref().map_or(1.0, |w| w.scale_factor() as f32);
                let point = self.to_image(pen::to_physical(position, scale));
                self.paint(point, pressure);
            }
            PenEvent::Move { .. } => {}
        }
    }

    /// # Sends the drawing to the GPU if it changed, then draws one frame.
    fn redraw(&mut self) {
        let (Some(gpu), Some(window)) = (&mut self.gpu, &self.window) else {
            return;
        };

        if self.canvas_dirty {
            gpu.update_canvas(self.drawing.take_dirty_rgba8());
            self.canvas_dirty = false;
        }

        gpu.set_viewport(&self.viewport);
        self.stats.begin_frame();
        let start = Instant::now();
        gpu.render(BACKGROUND, window, |ctx| self.stats.show(ctx));
        self.stats.end_frame(start.elapsed());
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }

        match create_window_and_gpu(event_loop, &self.drawing) {
            Ok((window, gpu)) => {
                self.tablet = connect_tablet(&window);
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
        if self.interface_takes(&event) {
            return;
        }

        self.track_pan_trigger(&event);

        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(size) => {
                if let (Some(gpu), Some(window)) = (&mut self.gpu, &self.window) {
                    gpu.resize(size.width, size.height);
                    window.request_redraw();
                }
            }
            WindowEvent::CursorMoved { position, .. } => {
                let position = (position.x as f32, position.y as f32);
                if self.pan_to(position) {
                    return;
                }
                self.cursor = self.to_image(position);
                if self.pressed {
                    self.paint(self.cursor, FULL_PRESSURE);
                }
            }
            WindowEvent::MouseInput {
                state,
                button: MouseButton::Left,
                ..
            } => match state {
                ElementState::Pressed if self.view_input.is_panning() => {}
                ElementState::Pressed => {
                    self.pressed = true;
                    self.paint(self.cursor, FULL_PRESSURE);
                }
                ElementState::Released => self.release(),
            },
            WindowEvent::RedrawRequested => self.redraw(),
            _ => {}
        }
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        let Some(tablet) = &mut self.tablet else {
            return;
        };

        for event in pen::poll(tablet) {
            self.handle_pen(event);
        }
        event_loop.set_control_flow(ControlFlow::WaitUntil(Instant::now() + PEN_POLL));
    }
}

/// # Connects to the graphics tablet of the window.
///
/// Without a tablet the program still works with the mouse, so a failure is only reported.
fn connect_tablet(window: &Arc<Window>) -> Option<Manager> {
    match Builder::default().build_shared(window) {
        Ok(manager) => Some(manager),
        Err(error) => {
            eprintln!("rimp: no tablet support: {error}");
            None
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
