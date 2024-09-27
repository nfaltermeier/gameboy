extern crate bitflags;
extern crate bitmatch;

mod constants;
mod memory;
mod memory_controllers;
mod opcodes;
mod operations;
mod system;

use memory::MemoryController;
use memory_controllers::basic_memory::BasicMemory;
use opcodes::process_instruction;

fn main() {
    /*
     * https://archive.org/details/GameBoyProgManVer1.1/page/n7/mode/2up?view=theater
     * general todo:
     * system registers pg 17, initial values pg 23, pg 268
     * interrupts see page 24
     * finish and test instructions
     * MBCs pg 215
     * display pg 48
     * color display for gbc?
     * sound pg 79
     * input (including reset switch)
     * cycle clock .954us or on gbc .477us switchable
     * read ROM
     * serial communication?
     * system startup pg 23, 127
     * persistent saves
     */ 

    let mut m = BasicMemory::default();
    m.r.set_flags_unchecked(0xFF);
    println!("result 1: {:#b}", m.r.f.bits());
    println!("result 2: {}", (-1) >> 1);
    println!("result 3: {}", 0xFFu8 >> 1);
    println!("result 4: {}", (0xFFu8 as i8) >> 1);
    println!("result 5: {:?}", 1_u16.overflowing_sub(2));
    println!("result 6: {}", (0x80u8) << 1);
    println!("result 7: {}", ((0xFFu8 as i8) >> 1) as u8);

    m.r.pc = 0;
    m.write_8(0, 0b00_001_010);
    process_instruction(&mut m);
}
