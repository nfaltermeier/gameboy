use std::{thread, time::{self, Duration, Instant}};

use crate::{constants::*, memory::MemoryController, memory_controllers::basic_memory::BasicMemory, opcodes::{process_instruction, u16_to_u8s}};

pub fn boot(rom: Vec<u8>) {
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

    run_loop(&mut *mem);
}

fn run_loop(mem: &mut dyn MemoryController) {
    let mut ime_actually_enabled = false;
    let mut ime_actually_enable_next = false;
    let mut time_next_instruction = Instant::now();
    let mut time_next_frame = Instant::now();

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

        if now >= time_next_frame {

        }
    }
}

fn wait_cycles(cycles: u64, next_instruction: &mut Instant, now: &Instant) {
    *next_instruction = match now.checked_add(Duration::from_nanos(954 * cycles)) {
        Some(i) => i,
        None => panic!("Could not set instant for next instruction"),
    }
}
