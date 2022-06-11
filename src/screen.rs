use pixels::{Pixels, SurfaceTexture};
use winit::{event_loop::EventLoop, window::Window, window::WindowBuilder};
use winit::dpi::LogicalSize;

pub fn create_window(event_loop: &EventLoop<()>, width: u32, height: u32) -> Window {
    let size = LogicalSize::new(width as f64, height as f64);
    WindowBuilder::new()
        .with_title("Mandelbrot-set")
        .with_inner_size(size)
        .with_min_inner_size(size)
        .build(&event_loop)
        .unwrap()
}


pub fn create_pixels(window: &Window, width: u32, height: u32) -> Pixels {
    let pixels = match {
        let window_size = window.inner_size();
        let surface_texture = SurfaceTexture::new(window_size.width, window_size.height, &window);
        Pixels::new(width, height, surface_texture)
    } {
        Ok(p) => {p},
        Err(_) => {
            panic!("Error creating pixels");
        }
    };
    pixels
}