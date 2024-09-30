use std::time::Instant;

use macroquad::prelude::*;

pub const SCREEN_WIDTH: u16 = 160;
pub const SCREEN_HEIGHT: u16 = 144;

pub struct LCD {
    image: Image,
    texture: Texture2D,
}

// ref: https://github.com/not-fl3/macroquad/blob/master/examples/life.rs
impl LCD {
    pub fn new() -> Self {
        let mut lcd = LCD {
            image: Image::gen_image_color(SCREEN_WIDTH, SCREEN_HEIGHT, WHITE),
            texture: Texture2D::empty()
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

        next_frame().await;
    }
}
