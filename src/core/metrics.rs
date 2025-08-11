use std::{collections::VecDeque, time::Instant};

pub struct EngineMetrics {
    frame_time: VecDeque<f32>,
    fps: f32,
    render_objects_count: usize,
    last_update: Instant,
}

impl EngineMetrics {
    pub fn new() -> Self {
        Self {
            frame_time: VecDeque::with_capacity(60),
            fps: 0.0,
            render_objects_count: 0,
            last_update: Instant::now(),
        }
    }

    pub fn update(&mut self, dt: f32, object_count: usize) {
        self.frame_time.push_back(dt);
        if self.frame_time.len() > 60 {
            self.frame_time.pop_front();
        }

        let avg_frame_time: f32 =
            self.frame_time.iter().sum::<f32>() / self.frame_time.len() as f32;

        self.fps = 1.0 / avg_frame_time;
        self.render_objects_count = object_count;
    }

    pub fn get_fps(&self) -> f32 {
        self.fps
    }

    pub fn get_frame_time_ms(&self) -> f32 {
        self.frame_time.back().unwrap_or(&0.0) * 1000.0
    }

    pub fn get_object_count(&self) -> usize {
        self.render_objects_count
    }

    pub fn check_performance(&self) {
        if self.fps < 30.0 {
            log::warn!("Low FPS: {:.1} fps", self.fps);
        }
        if self.get_frame_time_ms() > 33.0 {
            log::warn!("High frame time: {:.1}ms", self.get_frame_time_ms());
        }
    }
}
