use std::sync::mpsc::RecvError;

use pixels::{Pixels, SurfaceTexture};
use winit::{event_loop::EventLoop, window::Window, window::WindowBuilder};
use winit::dpi::LogicalSize;

use crate::work::WorkData;

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

pub fn mutate_frame_with_result(frame: &mut [u8], data_transfer_result: Result<(WorkData, Vec<u8>), RecvError>) {
    match data_transfer_result {
        Ok((data, result)) => {
            let start = data.start as usize;
            let end = (data.start + data.size) as usize;

            let frame_slice: &mut [u8] = &mut frame[start..end];

            frame_slice.copy_from_slice(&result);
        },
        Err(_) => {
            panic!("WorkQueue::get_work() tried to lock a poisoned mutex");
        }
    }
}