use num::Complex;
use winit::dpi::LogicalSize;

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

fn render(pixels:&mut [u8], bounds: (usize, usize)) {
    let half_bounds_0 = (bounds.0 as f64)/2.0;
    let half_bounds_1 = (bounds.1 as f64)/2.0;

    for row in 0..bounds.1 {
        for column in 0..bounds.0 {
            let point = Complex {
                re: (half_bounds_0 - row as f64)/100.0,
                im: (half_bounds_1 - column as f64)/100.0
            };
            pixels[row * bounds.0 + column] =
                match escape_time(point, 255) {
                    None => 0,
                    Some(count) => 255 - count as u8
                };
        }
    }
}

use winit::{
    event::{Event, WindowEvent, VirtualKeyCode},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};
use winit_input_helper::WinitInputHelper;
use log::error;
use pixels::{Error, Pixels, SurfaceTexture};

fn main() -> Result<(), Error> {
    println!("Hello, world!");

    let bounds = (1000, 1000);
    
    let size = LogicalSize::new(bounds.0 as f64, bounds.1 as f64);

    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title("mandelbrot-set")
        .with_inner_size(size)
        .with_min_inner_size(size)
        .build(&event_loop)
        .unwrap();

    let mut input = WinitInputHelper::new();

    let mut pixels = {
        let window_size = window.inner_size();
        let surface_texture = SurfaceTexture::new(window_size.width, window_size.height, &window);
        Pixels::new(bounds.0 as u32, bounds.1 as u32, surface_texture)?
    };

    render(pixels.get_frame(), bounds);

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        // Draw the current frame
        if let Event::RedrawRequested(_) = event {        
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
        }
    });
}
