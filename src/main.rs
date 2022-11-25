extern crate bitflags;

mod memory;
mod opcodes;
mod operations;

use memory::Memory;

fn main() {
    let mut m = Memory::default();
    println!("result: {}", (-1) >> 1);
    println!("result: {}", 0xFFu8 >> 1);
}
