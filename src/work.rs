use std::{sync::{Arc, Mutex}, collections::VecDeque};
use num::Complex;

use crate::mandelbrot::escape_time;

#[derive(Copy, Clone)]
pub struct WorkData {
    pub start: i64,
    pub size: i64,
    pub camera_zoom: f64,
    pub camera_x: f64,
    pub camera_y: f64
}

#[derive(Clone)]
pub struct WorkQueue<T: Send + Copy> {
    inner: Arc<Mutex<VecDeque<T>>>,
}

impl<T: Send + Copy> WorkQueue<T> {
    pub fn new() -> Self {
        Self { inner: Arc::new(Mutex::new(VecDeque::new())) }
    }

    pub fn get_work(&self) -> Option<T> {
        let maybe_queue = self.inner.lock();

        if let Ok(mut queue) = maybe_queue {
            queue.pop_front()
        } else {
            panic!("WorkQueue::get_work() tried to lock a poisoned mutex");
        }
    }

    pub fn add_work(&self, work: T) -> usize {
        if let Ok(mut queue) = self.inner.lock() {
            queue.push_back(work);

            queue.len()
        } else {
            panic!("WorkQueue::add_work() tried to lock a poisoned mutex");
        }
    }
}

pub fn thread_work(data: WorkData, width: usize) -> Vec<u8> {
    let mut out_vec = vec![0; data.size as usize];
    for (i, pixel) in out_vec.chunks_exact_mut(4).enumerate() {
        let real_i = i + (data.start/4) as usize;

        let x = (real_i % width) as f64 + data.camera_x;
        let y = (real_i / width) as f64 + data.camera_y;

        let point = Complex {
            re: ((((width as f64)/2.0) - x as f64))/data.camera_zoom,
            im: ((((width as f64)/2.0) - y as f64))/data.camera_zoom
        };

        let color = match escape_time(point, 255) {
            None => 0,
            Some(count) => 255 - count as u8
        };

        let rgba = [color, color, color, color];

        pixel.copy_from_slice(&rgba);
    }

    out_vec
}