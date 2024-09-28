extern crate bitflags;
extern crate bitmatch;

mod constants;
mod memory;
mod memory_controllers;
mod opcodes;
mod operations;
mod system;

use std::{env, fs};

use system::boot;

fn main() {
    /*
     * https://archive.org/details/GameBoyProgManVer1.1/page/n7/mode/2up?view=theater
     * general todo:
     * ✓ system registers pg 17, initial values pg 23, pg 268
     * ✓ interrupts see page 24
     * divider timer p25
     * main timer p25
     * finish and test instructions
     * MBCs pg 215
     * display pg 48
     * color display for gbc?
     * sound pg 79
     * input (including reset switch)
     * ✓ cycle clock .954us or on gbc .477us switchable
     * ✓ read ROM
     * serial communication?
     * system startup pg 23, 127
     * persistent saves
     */

    let args: Vec<String> = env::args().collect();
    let rom: Vec<u8>;
    dbg!(&args);
    if args.len() > 1 {
        match fs::read(&args[1]) {
            Ok(data) => rom = data,
            Err(err) => panic!("Failed reading rom file: {}", err)
        };
    } else {
        panic!("You must specify a rom path in the first argument")
    }

    boot(rom);
}
