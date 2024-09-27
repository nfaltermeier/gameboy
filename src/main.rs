extern crate bitflags;
extern crate bitmatch;

mod memory;
mod opcodes;
mod operations;

use memory::Memory;

use crate::opcodes::process_instruction;

fn main() {
    /*
     * general todo:
     * system registers pg 17, initial values pg 23, pg 268
     * interrupts see page 24
     * finish and test instructions
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

    let mut m = Memory::default();
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
