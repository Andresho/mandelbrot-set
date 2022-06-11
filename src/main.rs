#![deny(clippy::all)]
#![forbid(unsafe_code)]

use log::error;
use pixels::{Error};
use winit::event::{VirtualKeyCode};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::{Window};
use winit_input_helper::WinitInputHelper;
use work::thread_work;
use std::sync::mpsc::{channel};
use std::thread::{self, JoinHandle};

mod sync_flags;
mod camera;
mod work;
mod mandelbrot;
mod screen;

const MAX_WORKER: usize = 8;

const WIDTH: u32 = 800;
const HEIGHT: u32 = 800;

fn main() -> Result<(), Error> {
    let event_loop = EventLoop::new();
    let mut input = WinitInputHelper::new();
    let window = screen::create_window(&event_loop, WIDTH, HEIGHT);

    let (
        mut more_jobs_state_sender,
        more_jobs_state_receiver
    ) = sync_flags::new_syncflag(true);

    let mut camera = camera::Camera::new();

    let mut work_queue = work::WorkQueue::<work::WorkData>::new();

    let thread_work_queue = work_queue.clone();
    let _handle = thread::spawn(move || {
        create_threads(window, more_jobs_state_receiver, thread_work_queue);
    });

    create_works(&mut work_queue, camera.get_state());

    event_loop.run(move |event, _, control_flow| {
        if input.update(&event) {
            if input.key_pressed(VirtualKeyCode::Escape) || input.quit() {
                *control_flow = ControlFlow::Exit;
                more_jobs_state_sender.set(false).unwrap();
                return;
            }
            if input.key_pressed(winit::event::VirtualKeyCode::Left) {
                camera.go_left();
                create_works(&mut work_queue, camera.get_state());
            } else if input.key_pressed(winit::event::VirtualKeyCode::Right) {
                camera.go_right();
                create_works(&mut work_queue, camera.get_state());
            } else if input.key_pressed(winit::event::VirtualKeyCode::Up) {
                camera.go_up();
                create_works(&mut work_queue, camera.get_state());
            } else if input.key_pressed(winit::event::VirtualKeyCode::Down) {
                camera.go_down();
                create_works(&mut work_queue, camera.get_state());
            } else if input.key_pressed(winit::event::VirtualKeyCode::Z) {
                camera.zoom_in();
                create_works(&mut work_queue, camera.get_state());
            } else if input.key_pressed(winit::event::VirtualKeyCode::X) {
                camera.zoom_out();
                create_works(&mut work_queue, camera.get_state());
            }
        }
    });
}

fn create_works(work_queue: &mut work::WorkQueue::<work::WorkData>, camera_state: camera::Camera) {
    let total_size = (WIDTH * HEIGHT) * 4;

    let calc_size = ((total_size as f64) / MAX_WORKER as f64) as i64;
    for i in 0..MAX_WORKER {
        let work = work::WorkData {
            start: (i * calc_size as usize) as i64,
            size: calc_size,
            camera_zoom: camera_state.camera_zoom,
            camera_x: camera_state.camera_x,
            camera_y: camera_state.camera_y
        };
        work_queue.add_work(work);
    }
}

fn create_threads(
    window: Window,
    more_jobs_state_receiver: sync_flags::SyncFlagReceiver,
    work_queue: work::WorkQueue::<work::WorkData>
) {
    let mut threads = Vec::<JoinHandle<()>>::new();

    let mut pixels = screen::create_pixels(&window, WIDTH, HEIGHT);

    let (
        results_sender,
        results_receiver
    ) = channel::<(work::WorkData, Vec<u8>)>();

    for _thread_num in 0..MAX_WORKER {
        let thread_queue = work_queue.clone();

        let thread_results_sender = results_sender.clone();

        let thread_more_jobs_receiver = more_jobs_state_receiver.clone();

        let handle = thread::spawn(move || {
            while thread_more_jobs_receiver.get().unwrap() {

                if let Some(data) = thread_queue.get_work() {
                    let result = thread_work(data, WIDTH as usize);

                    match thread_results_sender.send((data, result)) {
                        Ok(_) => (),
                        Err(_) => { break; },
                    }

                }

                std::thread::yield_now();
            }
        });

        threads.push(handle);
    }

    while more_jobs_state_receiver.get().unwrap() {
        let data_transfer_result = results_receiver.recv();
        camera::Camera::mutate_frame_with_result(pixels.get_frame(), data_transfer_result);
        
        let _res = pixels
            .render()
            .map_err(|e| error!("pixels.render() failed: {}", e))
            .is_err();
        window.request_redraw();
    }

    for handle in threads {
        handle.join().unwrap();
    }
}
