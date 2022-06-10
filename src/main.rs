#![deny(clippy::all)]
#![forbid(unsafe_code)]

use log::error;
use pixels::{Error, Pixels, SurfaceTexture};
use winit::dpi::LogicalSize;
use winit::event::{Event, VirtualKeyCode};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::{WindowBuilder, Window};
use winit_input_helper::WinitInputHelper;
use std::sync::mpsc::{channel, Sender, Receiver, RecvError};
use std::thread::{self, JoinHandle};
use num::Complex;
use std::sync::Mutex;
use std::collections::VecDeque;
use std::sync::Arc;


const MAX_WORKER: usize = 2;

const WIDTH: u32 = 800;
const HEIGHT: u32 = 800;

struct Camera {
    work_queue: WorkQueue<WorkData>,
    threads: Vec<JoinHandle<()>>,
    camera_zoom: f64,
    camera_x: f64,
    camera_y: f64,
    velocity_x: i16,
    velocity_y: i16,
    results_sender: Sender<(WorkData, Vec<u8>)>,
    results_receiver: Receiver<(WorkData, Vec<u8>)>,
    more_jobs_state_sender: SyncFlagSender, 
    more_jobs_state_receiver: SyncFlagReceiver
}

#[derive(Copy, Clone)]
struct WorkData {
    start: i64, 
    size: i64, 
    camera_zoom: f64, 
    camera_x: f64, 
    camera_y: f64
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

    let (
        mut more_jobs_state_sender, 
        more_jobs_state_receiver
    ) = new_syncflag(true);

    let mut camera = Camera::new();

    let mut work_queue = WorkQueue::<WorkData>::new();

    let thread_work_queue = work_queue.clone();
    let handle = thread::spawn(move || {
        create_threads(window, more_jobs_state_receiver, thread_work_queue);
    });

    create_works(&mut work_queue);

    event_loop.run(move |event, _, control_flow| {
        if input.update(&event) {
            if input.key_pressed(VirtualKeyCode::Escape) || input.quit() {
                *control_flow = ControlFlow::Exit;
                more_jobs_state_sender.set(false).unwrap();
                return;
            }
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

fn create_works(work_queue: &mut WorkQueue::<WorkData>) {
    let total_size = (WIDTH * HEIGHT) * 4;

    let calc_size = ((total_size as f64) / MAX_WORKER as f64) as i64;
    for i in 0..MAX_WORKER {
        let work = WorkData { 
            start: (i * calc_size as usize) as i64, 
            size: calc_size, 
            camera_zoom: 300.0, 
            camera_x: 0.0, 
            camera_y: 0.0
        };
        work_queue.add_work(work);
    }
}

impl Camera {
    fn new() -> Self {
        let (results_sender, results_receiver) = channel();
        
        let (mut more_jobs_state_sender, more_jobs_state_receiver) = new_syncflag(true);

        Self {
            work_queue: WorkQueue::new(),
            threads: Vec::new(),
            camera_zoom: 0.6,
            camera_x: 200.0,
            camera_y: 0.0,
            velocity_x: 1,
            velocity_y: 1,
            results_sender: results_sender,
            results_receiver: results_receiver,
            more_jobs_state_sender: more_jobs_state_sender,
            more_jobs_state_receiver: more_jobs_state_receiver
        }
    }

    // fn update(&mut self) {
        
    // } 

    fn mutate_frame_with_result(frame: &mut [u8], data_transfer_result: Result<(WorkData, Vec<u8>), RecvError>) {
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

    fn work(data: WorkData) -> Vec<u8> {        
        let mut out_obj = vec![0; data.size as usize];
        for (i, pixel) in out_obj.chunks_exact_mut(4).enumerate() {
            let real_i = i + (data.start/4) as usize;
            
            let x = (real_i % WIDTH as usize) as f64 + data.camera_x;
            let y = (real_i / WIDTH as usize) as f64 + data.camera_y;

            let point = Complex {
                re: ((((WIDTH as f64)/2.0) - x as f64))/data.camera_zoom,
                im: ((((WIDTH as f64)/2.0) - y as f64))/data.camera_zoom
            };

            let color = match escape_time(point, 255) {
                None => 100,
                Some(count) => 255 - count as u8
            };

            let rgba = [color, color, color, color];

            pixel.copy_from_slice(&rgba);
        }

        out_obj
    }
}

fn create_threads(
    window: Window, 
    more_jobs_state_receiver: SyncFlagReceiver,
    work_queue: WorkQueue::<WorkData>
) {
    let mut threads = Vec::<JoinHandle<()>>::new();

    let mut pixels = match {
        let window_size = window.inner_size();
        let surface_texture = SurfaceTexture::new(window_size.width, window_size.height, &window);
        Pixels::new(WIDTH, HEIGHT, surface_texture)
    } {
        Ok(p) => {p},
        Err(_) => {
            panic!("Error creating pixels");
        }
    };

    let (
        results_sender, 
        results_receiver
    ) = channel::<(WorkData, Vec<u8>)>();

    for thread_num in 0..MAX_WORKER {
        let thread_queue = work_queue.clone();

        let thread_results_sender = results_sender.clone();

        let thread_more_jobs_receiver = more_jobs_state_receiver.clone();

        let handle = thread::spawn(move || {
            while thread_more_jobs_receiver.get().unwrap() {

                if let Some(data) = thread_queue.get_work() {
                    let result = Camera::work(data);

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
        Camera::mutate_frame_with_result(pixels.get_frame(), data_transfer_result);
        let res = pixels
            .render()
            .map_err(|e| error!("pixels.render() failed: {}", e))
            .is_err();
        window.request_redraw();
    }

    for handle in threads {
        handle.join().unwrap();
    }
}
#[derive(Clone)]
struct WorkQueue<T: Send + Copy> {
    inner: Arc<Mutex<VecDeque<T>>>,
}

impl<T: Send + Copy> WorkQueue<T> {
    fn new() -> Self { 
        Self { inner: Arc::new(Mutex::new(VecDeque::new())) } 
    }


    fn get_work(&self) -> Option<T> {
        let maybe_queue = self.inner.lock();

        if let Ok(mut queue) = maybe_queue {
            queue.pop_front()
        } else {
            panic!("WorkQueue::get_work() tried to lock a poisoned mutex");
        }
    }

    fn add_work(&self, work: T) -> usize {
        if let Ok(mut queue) = self.inner.lock() {
            queue.push_back(work);

            queue.len()
        } else {
            panic!("WorkQueue::add_work() tried to lock a poisoned mutex");
        }
    }
}

struct SyncFlagSender {
    inner: Arc<Mutex<bool>>,
}

impl SyncFlagSender {
    fn set(&mut self, state: bool) -> Result<(), ()> {
        if let Ok(mut v) = self.inner.lock() {
            *v = state;
            Ok(())
        } else {
            Err(())
        }
    }
}

#[derive(Clone)]
struct SyncFlagReceiver {
    inner: Arc<Mutex<bool>>,
}

impl SyncFlagReceiver {
    fn get(&self) -> Result<bool, ()> {
        if let Ok(v) = self.inner.lock() {
            Ok(*v)
        } else {
            Err(())
        }
    }
}

fn new_syncflag(initial_state: bool) -> (SyncFlagSender, SyncFlagReceiver) {
    let state = Arc::new(Mutex::new(initial_state));
    let tx = SyncFlagSender { inner: state.clone() };
    let rx = SyncFlagReceiver { inner: state.clone() };

    return (tx, rx);
}
