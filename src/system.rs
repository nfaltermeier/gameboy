use std::{thread, time};

use crate::{constants::*, memory::MemoryController, memory_controllers::basic_memory::BasicMemory, opcodes::{process_instruction, u16_to_u8s}};

struct ImeData {
    ime_actually_enabled: bool,
    actually_enable_next: bool,
}

pub fn boot(rom: Vec<u8>) {
    let mbc_type = rom[0x147];
    let mut mem: Box<dyn MemoryController>;
    let mut ime_data = ImeData {
        ime_actually_enabled: false,
        actually_enable_next: false
    };

    match mbc_type {
        0 => {
            mem = Box::new(BasicMemory::new(rom));
        },
        _ => todo!("Need to implement more mbc types. Tried to use: {:#x}", mbc_type)
    }

    mem.r().sp = ADDRESS_STACK_START;
    mem.write_8(ADDRESS_LCDC, 0x83);
    *mem.ime() = false;

    loop {
        run_next(&mut *mem, &mut ime_data);
    }
}

fn run_next(mem: &mut dyn MemoryController, ime_data: &mut ImeData) {
    let mut cycles: u64 = 0;

    if ime_data.ime_actually_enabled {
        // Check interrupts
        let interrupt_requests = mem.read_8(ADDRESS_IF);
        let interrupt_enabled = mem.read_8(ADDRESS_IE);

        for i in 0..5 {
            if (interrupt_requests & (1 << i) != 0) && (interrupt_enabled & (1 << i) != 0) {
                *mem.ime() = false;
                mem.write_8(ADDRESS_IF, interrupt_requests & !(1 << i));

                let pc_vals = u16_to_u8s(mem.r().pc);
                mem.write_8(mem.r_i().sp - 1, pc_vals.0);
                mem.write_8(mem.r_i().sp - 2, pc_vals.1);
                mem.r().sp -= 2;

                mem.r().pc = ADDRESS_FIRST_INTERRUPT_HANDLER + i * 0x08;
                cycles += 5;
                break;
            }
        }
    }

    wait_cycles(&mut cycles);

    cycles = process_instruction(mem);

    wait_cycles(&mut cycles);

    if !*mem.ime() {
        ime_data.ime_actually_enabled = false;
        ime_data.actually_enable_next = false;
    } else if !ime_data.ime_actually_enabled {
        if ime_data.actually_enable_next {
            ime_data.ime_actually_enabled = true;
            ime_data.actually_enable_next = false;
        } else {
            ime_data.actually_enable_next = true;
        }
    }
}

fn wait_cycles(cycles: &mut u64) {
    if *cycles > 0 {
        thread::sleep(time::Duration::from_nanos(954 * *cycles));
        *cycles = 0;
    }
}
