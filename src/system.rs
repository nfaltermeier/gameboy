use std::{collections::VecDeque, thread, time::{self, Duration, Instant}};

use morton_encoding::morton_encode;

use crate::{constants::*, lcd::LCD, memory::MemoryController, memory_controllers::basic_memory::BasicMemory, model::model_render::{OamScanData, PixelRenderData}, opcodes::{process_instruction, u16_to_u8s}};

pub async fn boot(rom: Vec<u8>) {
    let mbc_type = rom[0x147];
    let mut mem: Box<dyn MemoryController>;

    match mbc_type {
        0 => {
            mem = Box::new(BasicMemory::new(rom));
        },
        _ => todo!("Need to implement more mbc types. Tried to use: {:#x}", mbc_type)
    }

    mem.r().sp = ADDRESS_STACK_START;
    mem.write_8(ADDRESS_LCDC, 0x83);
    *mem.ime() = false;

    run_loop(&mut *mem).await;
}

async fn run_loop(mem: &mut dyn MemoryController) {
    let mut ime_actually_enabled = false;
    let mut ime_actually_enable_next = false;
    let mut time_next_instruction = Instant::now();

    let mut time_next_ppu = Instant::now();
    let mut dots_left = 1;
    let mut oam_scan = OamScanData { current_object: 0, objects: VecDeque::new() };
    let mut pixel_render = PixelRenderData::new();
    let mut first_dot_after_switch = false;
    let mut lcd = LCD::new();

    loop {
        let now = Instant::now();
        let mut interrupt_triggered = false;

        if now >= time_next_instruction {
            if ime_actually_enabled {
                // Check interrupts
                let interrupt_requests = mem.read_8(ADDRESS_IF);
                let interrupt_enabled = mem.read_8(ADDRESS_IE);

                for i in 0..5 {
                    let interrupt_can_start = interrupt_requests & interrupt_enabled;
                    if interrupt_can_start & (1 << i) != 0 {
                        *mem.ime() = false;
                        mem.write_8(ADDRESS_IF, interrupt_requests & !(1 << i));

                        let pc_vals = u16_to_u8s(mem.r().pc);
                        mem.write_8(mem.r_i().sp - 1, pc_vals.0);
                        mem.write_8(mem.r_i().sp - 2, pc_vals.1);
                        mem.r().sp -= 2;

                        mem.r().pc = ADDRESS_FIRST_INTERRUPT_HANDLER + i * 0x08;
                        wait_cycles(5, &mut time_next_instruction, &now);
                        interrupt_triggered = true;
                        break;
                    }
                }
            }

            if !interrupt_triggered {
                let cycles = process_instruction(mem);
    
                wait_cycles(cycles, &mut time_next_instruction, &now);
            }

            if !*mem.ime() {
                ime_actually_enabled = false;
                ime_actually_enable_next = false;
            } else if !ime_actually_enabled {
                if ime_actually_enable_next {
                    ime_actually_enabled = true;
                    ime_actually_enable_next = false;
                } else {
                    ime_actually_enable_next = true;
                }
            }
        }

        if now >= time_next_ppu {
            dots_left -= 1;
            let reset_first_dot_flag = first_dot_after_switch;

            let stat = mem.read_8(ADDRESS_STAT);
            let ppu_mode = stat & 0b00000011;

            if crate::debug::DEBUG_PRINT_PPU {
                println!("dots_left: {}", dots_left);
                println!("ppu_mode: {}", ppu_mode);
            }

            match ppu_mode {
                PPU_MODE_OAM_SCAN => {
                    if first_dot_after_switch {
                        oam_scan.current_object = 0;
                        oam_scan.objects.clear();
                    }

                    if dots_left % 2 == 0 && oam_scan.objects.len() < 10 {
                        let lcdc = mem.read_8(ADDRESS_LCDC);
                        let obj_height: u8 = if lcdc & LCDC_OBJ_SIZE != 0 { 16 } else { 8 };
                        let ly = mem.read_8(ADDRESS_LY);

                        let obj_addr = ADDRESS_OAM_START + 4 * oam_scan.current_object;
                        let obj_y = mem.read_8(obj_addr);

                        if obj_on_screen(ly, obj_y, obj_height) {
                            oam_scan.objects.push_back(obj_addr);
                        }

                        oam_scan.current_object += 1;
                    }

                    if dots_left == 0 {
                        // should be 172? Depends on how the delays are added later.
                        dots_left = 160;
                        // + 1 will change mode from 2 to 3
                        mem.write_8(ADDRESS_STAT, stat + 1); 
                        first_dot_after_switch = true;
                    }
                }
                PPU_MODE_RENDER_PIXEL => {
                    if first_dot_after_switch {
                        pixel_render.reset();
                    }

                    if pixel_render.x < 160 {
                        let lcdc = mem.read_8(ADDRESS_LCDC);
                        let ly = mem.read_8(ADDRESS_LY);
                        // todo: use palettes
                        if !pixel_render.background_queue.len() >= 8 {
                            let scx = mem.read_8(ADDRESS_SCX);
                            let scy = mem.read_8(ADDRESS_SCY);

                            // todo: pandocs suggest this should be broken out into an operation over multiple dots
                            let tiledata_index_address: u16;
                            let window_enabled = (lcdc & LCDC_WINDOW_ENABLE) != 0;
                            if window_enabled {
                                todo!("Window not implemented")
                            } else {
                                let tilemap_address = if (lcdc & LCDC_BG_TILEMAP) != 0 { ADDRESS_TILEMAP_2 } else { ADDRESS_TILEMAP_1 };
                                // https://gbdev.io/pandocs/pixel_fifo.html gives this Y coord code. Doesn't seem right at all. Misintepreting the docs?
                                // let x = ((scx / 8) + pixel_render.tile_x) % 32;
                                // let y = ly.wrapping_add(scy);
                                let x = ((scx / 8) + pixel_render.tile_x / 8) % 32;
                                let y = scy / 8 + ly / 8;

                                tiledata_index_address = tilemap_address + x as u16 + y as u16 * 32;
                            }

                            let mut tile_data_index = mem.read_8(tiledata_index_address);
                            let tile_data_address_mode_easy = (lcdc & LCDC_BG_AND_WINDOW_TILEDATA) != 0;
                            let tile_data_address: u16;
                            if tile_data_address_mode_easy {
                                tile_data_address = ADDRESS_TILEDATA_1 + tile_data_index as u16 * 16;
                            } else {
                                if tile_data_index > 127 {
                                    tile_data_index -= 128;
                                } else {
                                    tile_data_index += 128;
                                }

                                // ADDRESS_TILEDATA_2 should be 0x8800 not 0x9000 like documentation will give because
                                // this code is not using signed numbers for the index.
                                tile_data_address = ADDRESS_TILEDATA_2 + tile_data_index as u16 * 16;
                            }

                            let tile_low = mem.read_8(tile_data_address);
                            let tile_high = mem.read_8(tile_data_address + 1);

                            let mut all_pixel_data = morton_encode([tile_high, tile_low]);

                            for _i in 0..8 {
                                let pixel = ((all_pixel_data & 0xC000) >> 14) as u8;
                                pixel_render.background_queue.push_back(pixel);
                                all_pixel_data = all_pixel_data << 2;
                            }
                        }

                        if oam_scan.objects.len() > 0 {
                            let obj_addr = *oam_scan.objects.front().unwrap();
                            let obj_x = mem.read_8(obj_addr + 1);

                            let tall_tiles = (lcdc & LCDC_OBJ_SIZE) != 0;
                            if tall_tiles {
                                todo!("Implement 8x16 tile addressing")
                            }

                            // account for 8 pixel offset compared to screen
                            if pixel_render.x + 8 == obj_x {
                                oam_scan.objects.pop_front().unwrap();
                                let obj_y = mem.read_8(obj_addr);
                                let obj_index = mem.read_8(obj_addr + 2);
                                let obj_attrs = mem.read_8(obj_addr + 3);
                                let row_in_tile_offset = obj_y + if tall_tiles { 16 } else { 8 } - 16 - ly;
                                let tile_data_address = ADDRESS_TILEDATA_1 + obj_index as u16 * 16 + row_in_tile_offset as u16 * 2;

                                // todo: check x and y flip
                                let tile_low = mem.read_8(tile_data_address);
                                let tile_high = mem.read_8(tile_data_address + 1);

                                let mut all_pixel_data = morton_encode([tile_high, tile_low]);

                                let obj_queue_len = pixel_render.obj_queue.len();
                                pixel_render.obj_queue.make_contiguous();
                                let queue_contents = pixel_render.obj_queue.as_mut_slices().0;

                                let priority_data = if (obj_attrs & 1 << 7) != 0 { 4 } else { 0 };
                                for i in 0..8 {
                                    let pixel = ((all_pixel_data & 0xC000) >> 14) as u8 | priority_data;

                                    if i < obj_queue_len {
                                        // pixel in queue is transparent or behind bg
                                        if queue_contents[i] & 3 == 0 || queue_contents[i] & 4 != 0 {
                                            queue_contents[i] = pixel;
                                        }
                                    } else {
                                        pixel_render.background_queue.push_back(pixel);
                                    }

                                    all_pixel_data = all_pixel_data << 2;
                                }
                            }
                        }

                        // actually draw a pixel now
                        let bg = pixel_render.background_queue.pop_front();
                        let obj = pixel_render.obj_queue.pop_front();
                        let bg_disabled = lcdc & LCDC_BG_WINDOW_ENABLE != 0;
                        match (bg, obj) {
                            (Some(bgv), Some(objv)) => {
                                let obj_low_priority = objv & 4 != 0;
                                let obj_color = objv & 3;
                                let bg_color = if bg_disabled { 0 } else { bgv };

                                if obj_color == 0 || obj_low_priority {
                                    lcd.draw_pixel(pixel_render.x, ly, bg_color);
                                } else {
                                    lcd.draw_pixel(pixel_render.x, ly, obj_color);
                                }
                            },
                            (Some(bgv), None) => {
                                let bg_color = if bg_disabled { 0 } else { bgv };
                                lcd.draw_pixel(pixel_render.x, ly, bg_color);
                            },
                            (None, Some(objv)) => {
                                let obj_low_priority = objv & 4 != 0;
                                let obj_color = objv & 3;

                                if obj_color != 0 && !obj_low_priority {
                                    lcd.draw_pixel(pixel_render.x, ly, obj_color);
                                }
                            },
                            _ => { }
                        }

                        pixel_render.x += 1;
                    }

                    if dots_left == 0 {
                        // transition to horiz blank
                        dots_left = 216;
                        // - 3 will change mode from 3 to 0
                        mem.write_8(ADDRESS_STAT, stat - 3);
                        first_dot_after_switch = true;
                    }
                }
                PPU_MODE_HORIZ_BLANK => {
                    if dots_left == 0 {
                        let ly = mem.read_8(ADDRESS_LY);
                        if ly == 143 {
                            // transition to vertical blank
                            dots_left = 456;
                            // + 1 will change mode from 0 to 1
                            mem.write_8(ADDRESS_STAT, stat + 1);
                        } else {
                            // transition to OAM scan
                            dots_left = 80;
                            // + 2 will change mode from 0 to 2
                            mem.write_8(ADDRESS_STAT, stat + 2);
                        }

                        mem.write_8(ADDRESS_LY, ly + 1);
                        first_dot_after_switch = true;
                    }
                }
                PPU_MODE_VERT_BLANK => {
                    if first_dot_after_switch {
                        lcd.show_frame().await;
                    }

                    if dots_left == 0 {
                        let ly = mem.read_8(ADDRESS_LY);
                        if ly == 153 {
                            // transition to OAM scan
                            dots_left = 80;
                            // + 1 will change mode from 1 to 2
                            mem.write_8(ADDRESS_STAT, stat + 1);
                            mem.write_8(ADDRESS_LY, 0);
                            first_dot_after_switch = true;
                            lcd.start_new_frame();
                        } else {
                            dots_left = 456;
                            mem.write_8(ADDRESS_LY, ly + 1);
                        }
                    }
                }
                _ => panic!("Invalid ppu_mode")
            }

            // this has to go really fast... may need refactoring to keep up?
            time_next_ppu = now.checked_add(Duration::from_nanos(238)).unwrap();
            if reset_first_dot_flag {
                first_dot_after_switch = false;
            }
        }
    }
}

fn wait_cycles(cycles: u64, next_instruction: &mut Instant, now: &Instant) {
    *next_instruction = match now.checked_add(Duration::from_nanos(954 * cycles)) {
        Some(i) => i,
        None => panic!("Could not set instant for next instruction"),
    }
}

pub fn obj_on_screen(ly: u8, obj_y: u8, obj_height: u8) -> bool {
    let top_above = obj_y <= ly + 16;
    let bottom_below = obj_y + obj_height > ly + 16;
    top_above && bottom_below
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::obj_on_screen;

    #[rstest]
    #[case(0, 0, 8, false)]
    #[case(0, 0, 16, false)]
    #[case(0, 2, 8, false)]
    #[case(0, 2, 16, true)]
    #[case(0, 16, 8, true)]
    #[case(0, 16, 16, true)]
    #[case(143, 144, 8, false)]
    #[case(143, 144, 16, true)]
    #[case(143, 152, 8, true)]
    #[case(143, 152, 16, true)]
    #[case(143, 154, 8, true)]
    #[case(143, 154, 16, true)]
    #[case(143, 160, 8, false)]
    #[case(143, 160, 16, false)]
    fn obj_on_screen_test(#[case] ly: u8, #[case] obj_y: u8, #[case] obj_height: u8, #[case] expected_result: bool) {
        let result = obj_on_screen(ly, obj_y, obj_height);
        assert_eq!(expected_result, result);
    }
}
