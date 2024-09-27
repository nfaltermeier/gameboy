use crate::{memory::MemoryController, memory_controllers::basic_memory::BasicMemory, opcodes::process_instruction};

pub fn boot(rom: Vec<u8>) {
    let mbc_type = rom[0x147];
    let mut mem: Box<dyn MemoryController>;

    match mbc_type {
        0 => {
            mem = Box::new(BasicMemory::new(rom));
        },
        _ => todo!("Need to implement more mbc types")
    }

    loop {
        run_next(&mut *mem);
    }
}

pub fn run_next(mem: &mut dyn MemoryController) {
    // todo: The effect of ei is delayed by one instruction. This means that ei followed immediately by di does not allow any interrupts between them
    //       https://gbdev.io/pandocs/Interrupts.html
    if *mem.ime() {
        // Check interrupts
        let intF = mem.read_8(crate::constants::ADDRESS_IF);
    }

    process_instruction(mem);
}
