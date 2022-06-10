use std::{sync::mpsc::{RecvError}};

use crate::{work::{WorkData}};

pub struct Camera {
    pub camera_zoom: f64,
    pub camera_x: f64,
    pub camera_y: f64,
    pub velocity_x: f64,
    pub velocity_y: f64,
    pub velocity_zoom: f64,
}

impl Camera {
    pub fn new() -> Self {
        Self {
            camera_zoom: 200.0,
            camera_x: 100.0,
            camera_y: 50.0,
            velocity_x: 50.0,
            velocity_y: 50.0,
            velocity_zoom: 50.0
        }
    }

    pub fn go_up(&mut self) {
        self.camera_y = self.camera_y - self.velocity_y;
    }
    pub fn go_down(&mut self) {
        self.camera_y = self.camera_y + self.velocity_y;
    }
    pub fn go_left(&mut self) {
        self.camera_x = self.camera_x - self.velocity_x;
    }
    pub fn go_right(&mut self) {
        self.camera_x = self.camera_x + self.velocity_x;
    }
    pub fn zoom_in(&mut self) {
        self.camera_zoom = self.camera_zoom + self.velocity_zoom;
    }
    pub fn zoom_out(&mut self) {
        self.camera_zoom = self.camera_zoom - self.velocity_zoom;
    }
    pub fn get_state(&self) -> Camera {
        let state = Camera {
            camera_zoom: self.camera_zoom,
            camera_x: self.camera_x,
            camera_y: self.camera_y,
            velocity_x: self.velocity_x,
            velocity_y: self.velocity_y,
            velocity_zoom: self.velocity_zoom,
        };
        state
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
}
