use std::{collections::VecDeque, time::{Duration, Instant}};

use macroquad::prelude::*;

pub const SCREEN_WIDTH: u16 = 160;
pub const SCREEN_HEIGHT: u16 = 144;

pub struct Lcd {
    image: Image,
    texture: Texture2D,
    frame_times: VecDeque<Instant>,
    fps_ready: bool,
}

// ref: https://github.com/not-fl3/macroquad/blob/master/examples/life.rs
impl Lcd {
    pub fn new() -> Self {
        let mut lcd = Lcd {
            image: Image::gen_image_color(SCREEN_WIDTH, SCREEN_HEIGHT, WHITE),
            texture: Texture2D::empty(),
            frame_times: VecDeque::new(),
            fps_ready: false,
        };
        lcd.texture = Texture2D::from_image(&lcd.image);
        lcd
    }

    pub fn start_new_frame(&mut self) {
        clear_background(WHITE);
    }

    pub fn draw_pixel(&mut self, x: u8, y: u8, color: u8) {
        let show_color = match color {
            0 => WHITE,
            1 => LIGHTGRAY,
            2 => GRAY,
            3 => BLACK,
            _ => PINK,
        };

        self.image.set_pixel(x.into(), y.into(), show_color);
    }

    pub async fn show_frame(&mut self) {
        self.texture.update(&self.image);
        draw_texture(&self.texture, 0., 0., WHITE);

        if crate::debug::DEBUG_PRINT_FRAME_TIME {
            println!("Showing frame at {:?}", Instant::now());
        }

        if crate::debug::DEBUG_SHOW_FPS {
            let now = Instant::now();
            loop {
                match self.frame_times.front() {
                    Some(t) => {
                        if now.duration_since(*t) > Duration::from_secs(3) {
                            self.fps_ready = true;
                            self.frame_times.pop_front();
                            continue;
                        }
                    }
                    _ => {}
                }

                break;
            }

            self.frame_times.push_back(now);
            if self.fps_ready {
                let fps = self.frame_times.len() as f32 / 3.;
                println!("Current fps (3s avg): {}", fps);
            }
        }

        next_frame().await;
    }
}
