use std::{sync::mpsc::{RecvError}};

use crate::{work::{WorkData}};

pub struct Camera {
    camera_zoom: f64,
    camera_x: f64,
    camera_y: f64,
    velocity_x: i16,
    velocity_y: i16,
}

impl Camera {
    pub fn new() -> Self {
        Self {
            camera_zoom: 0.6,
            camera_x: 200.0,
            camera_y: 0.0,
            velocity_x: 1,
            velocity_y: 1,
        }
    }

    // fn update(&mut self) {

    // }

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
}
