#![deny(clippy::all)]
#![forbid(unsafe_code)]

use log::error;
use pixels::{Error, Pixels, SurfaceTexture};
use winit::dpi::LogicalSize;
use winit::event::{Event, VirtualKeyCode};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::WindowBuilder;
use winit_input_helper::WinitInputHelper;

use num::Complex;

const WIDTH: u32 = 800;
const HEIGHT: u32 = 800;

struct Camera {
    camera_zoom: f64,
    camera_x: f64,
    camera_y: f64,
    velocity_x: i16,
    velocity_y: i16,
}

fn main() -> Result<(), Error> {
    env_logger::init();
    let event_loop = EventLoop::new();
    let mut input = WinitInputHelper::new();
    let window = {
        let size = LogicalSize::new(WIDTH as f64, HEIGHT as f64);
        WindowBuilder::new()
            .with_title("Mandelbrot-set")
            .with_inner_size(size)
            .with_min_inner_size(size)
            .build(&event_loop)
            .unwrap()
    };

    let mut pixels = {
        let window_size = window.inner_size();
        let surface_texture = SurfaceTexture::new(window_size.width, window_size.height, &window);
        Pixels::new(WIDTH, HEIGHT, surface_texture)?
    };
    let mut camera = Camera::new();

    event_loop.run(move |event, _, control_flow| {
        if let Event::RedrawRequested(_) = event {
            camera.draw(pixels.get_frame());
            if pixels
                .render()
                .map_err(|e| error!("pixels.render() failed: {}", e))
                .is_err()
            {
                *control_flow = ControlFlow::Exit;
                return;
            }
        }

        // Handle input events
        if input.update(&event) {
            // Close events
            if input.key_pressed(VirtualKeyCode::Escape) || input.quit() {
                *control_flow = ControlFlow::Exit;
                return;
            }

            // Resize the window
            if let Some(size) = input.window_resized() {
                pixels.resize_surface(size.width, size.height);
            }

            // Update internal state and request a redraw
            camera.update();
            window.request_redraw();
        }
    });
}


fn escape_time(c: Complex<f64>, limit: usize) -> Option<usize> {
    let mut z = Complex { re: 0.0, im: 0.0 };
    for i in 0..limit {
        if z.norm_sqr() > 4.0 {
            return Some(i);
        }
        z = z * z + c;
    }
 
    None
}

impl Camera {
    fn new() -> Self {
        Self {
            camera_zoom: 0.6,
            camera_x: 0.0,
            camera_y: 0.0,
            velocity_x: 1,
            velocity_y: 1
        }
    }

    fn update(&mut self) {
        
    }

    fn draw(&self, frame: &mut [u8]) {
        let camera_zoom = 500.0 * self.camera_zoom;

        for (i, pixel) in frame.chunks_exact_mut(4).enumerate() {
            let x = (i % WIDTH as usize) as f64 + self.camera_x;
            let y = (i / WIDTH as usize) as f64 + self.camera_y;

            let point = Complex {
                re: ((((WIDTH as f64)/2.0) - x as f64))/camera_zoom,
                im: ((((WIDTH as f64)/2.0) - y as f64))/camera_zoom
            };

            let color = match escape_time(point, 255) {
                None => 100,
                Some(count) => 255 - count as u8
            };

            let rgba = [color, color, color, color];

            pixel.copy_from_slice(&rgba);
        }
    }
}
