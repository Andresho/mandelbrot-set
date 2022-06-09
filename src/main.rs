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
    work_queue: WorkQueue<i32>,
    threads: Vec<i32>,
    camera_zoom: f64,
    camera_x: f64,
    camera_y: f64,
    velocity_x: i16,
    velocity_y: i16,
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

use std::sync::mpsc::channel;
use std::thread;



impl Camera {
    fn new() -> Self {
        let (results_tx, results_rx) = channel();
    
        let (mut more_jobs_tx, more_jobs_rx) = new_syncflag(true);
    
        let mut threads = Vec::new();
    
        println!("Spawning {} workers.", MAX_WORKER);
        
        Self {
            work_queue: WorkQueue::new(),
            threads: Vec::new(),
            camera_zoom: 0.6,
            camera_x: 200.0,
            camera_y: 0.0,
            velocity_x: 1,
            velocity_y: 1
        }
    }

    fn update(&mut self) {
        
    } 

    fn create_threads(&self) {
        let queue = WorkQueue::<WorkData>::new();

        use std::sync::mpsc::channel;
        let (results_tx, results_rx) = channel();

        let (mut more_jobs_tx, more_jobs_rx) = new_syncflag(true);

        use std::thread;
        let mut threads = Vec::new();

        println!("Spawning {} workers.", MAX_WORKER);

        for thread_num in 0..MAX_WORKER {
            let thread_queue = queue.clone();

            let thread_results_tx = results_tx.clone();

            let thread_more_jobs_rx = more_jobs_rx.clone();

            let handle = thread::spawn(move || {
                let mut work_done = 0;

                while thread_more_jobs_rx.get().unwrap() {

                    if let Some(data) = thread_queue.get_work() {
                        let result = Camera::work(data);

                        work_done += 1;

                        match thread_results_tx.send((data, result)) {
                            Ok(_) => (),
                            Err(_) => { break; },
                        }

                    }

                    std::thread::yield_now();
                }

                println!("Thread {} did {} jobs.", thread_num, work_done);
            });

            threads.push(handle);
        }
    }

    fn work(data: WorkData) {
        let end = data.start + data.size/4;
        
        let mut out_obj = vec![0; (data.size as usize)];

        for (i, pixel) in out_obj.chunks_exact_mut(4).enumerate() {
            let x = (i % WIDTH as usize) as f64 + data.camera_x;
            let y = (i / WIDTH as usize) as f64 + data.camera_y;

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

        out_obj;
    }

    work_queue
    fn draw(&self, frame: &mut [u8]) {
        let split_size = frame.len() / 4;

        for i in 0..split_size {
            self.work_queue.add_work(work);
        }
    }

    // fn draw_old(&self, frame: &mut [u8]) {
    //     let camera_zoom = 500.0 * self.camera_zoom;

    //     for (i, pixel) in frame.chunks_exact_mut(4).enumerate() {
    //         let x = (i % WIDTH as usize) as f64 + self.camera_x;
    //         let y = (i / WIDTH as usize) as f64 + self.camera_y;

    //         let point = Complex {
    //             re: ((((WIDTH as f64)/2.0) - x as f64))/camera_zoom,
    //             im: ((((WIDTH as f64)/2.0) - y as f64))/camera_zoom
    //         };

    //         let color = match escape_time(point, 255) {
    //             None => 100,
    //             Some(count) => 255 - count as u8
    //         };

    //         let rgba = [color, color, color, color];

    //         pixel.copy_from_slice(&rgba);
    //     }
    // }
}





const MAX_WORKER: usize = 4;
use std::sync::Mutex;
use std::collections::VecDeque;
use std::sync::Arc;


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

struct SyncFlagTx {
    inner: Arc<Mutex<bool>>,
}

impl SyncFlagTx {
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
struct SyncFlagRx {
    inner: Arc<Mutex<bool>>,
}

impl SyncFlagRx {
    fn get(&self) -> Result<bool, ()> {
        if let Ok(v) = self.inner.lock() {
            Ok(*v)
        } else {
            Err(())
        }
    }
}

fn new_syncflag(initial_state: bool) -> (SyncFlagTx, SyncFlagRx) {
    let state = Arc::new(Mutex::new(initial_state));
    let tx = SyncFlagTx { inner: state.clone() };
    let rx = SyncFlagRx { inner: state.clone() };

    return (tx, rx);
}



// fn create_threads() {results_tx
//     use std::sync::mpsc::channel;
//     let (results_tx, results_rx) = channel();

//     use std::thread;
//     let mut threads = Vec::new();
// }


////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////


fn main_2() {
    let queue = WorkQueue::new();

    use std::sync::mpsc::channel;
    let (results_tx, results_rx) = channel();

    let (mut more_jobs_tx, more_jobs_rx) = new_syncflag(true);

    use std::thread;
    let mut threads = Vec::new();

    println!("Spawning {} workers.", MAX_WORKER);

    for thread_num in 0..MAX_WORKER {
        let thread_queue = queue.clone();

        let thread_results_tx = results_tx.clone();

        let thread_more_jobs_rx = more_jobs_rx.clone();

        let handle = thread::spawn(move || {
            let mut work_done = 0;

            while thread_more_jobs_rx.get().unwrap() {

                if let Some(work) = thread_queue.get_work() {
                    let result = fib(work);

                    work_done += 1;

                    match thread_results_tx.send((work, result)) {
                        Ok(_) => (),
                        Err(_) => { break; },
                    }

                }

                std::thread::yield_now();
            }

            println!("Thread {} did {} jobs.", thread_num, work_done);
        });

        threads.push(handle);
    }

    println!("Workers successfully started.");

    println!("Adding jobs to the queue.");
    
    let mut jobs_remaining = 0;
    let mut jobs_total = 0;

    for work in 0..90 {
        for _ in 0..100 {
            jobs_remaining = queue.add_work(work);
            jobs_total += 1;
        }
    }

    println!("Total of {} jobs inserted into the queue ({} remaining at this time).", 
             jobs_total,
             jobs_remaining);

    while jobs_total > 0 {
        match results_rx.recv() {
            Ok(_) => { jobs_total -= 1 },
            Err(_) => {panic!("All workers died unexpectedly.");}
        }
    }

    more_jobs_tx.set(false).unwrap();

    for handle in threads {
        handle.join().unwrap();
    }
}
