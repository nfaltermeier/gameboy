use std::collections::VecDeque;

pub struct OamScanData {
    pub current_object: u16,
    pub objects: VecDeque<u16>,
}

pub struct PixelRenderData {
    pub background_queue: VecDeque<u8>,
    pub obj_queue: VecDeque<u8>,
    pub x: u8,
    pub tile_x: u8,
}

impl PixelRenderData {
    pub fn new() -> Self {
        PixelRenderData {
            background_queue: VecDeque::new(),
            obj_queue: VecDeque::new(),
            x: 0,
            tile_x: 0,
        }
    }

    pub fn reset(&mut self) {
        self.background_queue.clear();
        self.obj_queue.clear();
        self.x = 0;
        self.tile_x = 0;
    }
}
