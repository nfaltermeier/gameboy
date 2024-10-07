use std::{slice::Iter, thread};
use std::sync::mpsc::Receiver;
use std::time::Duration;
use std::str::Split;
use std::sync::mpsc;
use std::io;

use indoc::indoc;

use crate::memory::MemoryController;
use crate::constants::*;

use super::{flags::DEBUG_TRACK_JUMPS, metrics::DebugMetrics};

enum CommandResult {
    PauseGame,
    None,
    ResumeGame,
}

pub struct DebugConsole {
    pause_next: bool,
    stat_next: bool,
    runto_address: Option<u16>,
    input: Receiver<String>,
    watch_addrs: Vec<u16>,
    break_pc_addrs: Vec<u16>,
}

impl DebugConsole {
    pub fn new() -> Self {
        DebugConsole {
            pause_next: false,
            stat_next: false,
            runto_address: None,
            input: Self::spawn_stdin_channel(),
            watch_addrs: vec![],
            break_pc_addrs: vec![],
        }
    }

    pub fn run(&mut self, mem: &mut dyn MemoryController, metrics: &mut DebugMetrics) {
        let mut pause = self.pause_next;
        self.pause_next = false;

        let pc = mem.r_i().pc;
        if self.runto_address.is_some_and(|addr| addr == pc) {
            println!("Breaking: runto reached. pc is {pc:#x}");
            self.runto_address = None;
            pause = true;
        }

        if !pause {
            for addr in &self.break_pc_addrs {
                if pc == *addr {
                    pause = true;
                    println!("Breaking, pc is {pc:#x}");
                    break;
                }
            }
        }

        if self.stat_next {
            println!("{:?}", mem.r_i());
            self.stat_next = false;
        }

        let mut first_print_watches = true;
        if pause {
            self.print_watches(mem);
            first_print_watches = false;
        }

        loop {
            match self.check_command(mem, metrics) {
                CommandResult::PauseGame => {
                    pause = true;
                }
                CommandResult::ResumeGame => {
                    pause = false;
                }
                CommandResult::None => {}
            }

            if !pause {
                break;
            }

            if first_print_watches {
                self.print_watches(mem);
                first_print_watches = false;
            }

            thread::sleep(Duration::from_millis(50));
        }
    }

    fn check_command(&mut self, mem: &mut dyn MemoryController, metrics: &mut DebugMetrics) -> CommandResult {
        match self.input.try_recv() {
            Ok(mut value) => {
                value = value.to_lowercase();
                let mut words = value.trim().split(' ');

                match words.next().unwrap_or("") {
                    "p" | "pause" => {
                        println!("Pausing");
                        CommandResult::PauseGame
                    }
                    "g" | "go" => {
                        println!("Going");
                        CommandResult::ResumeGame
                    }
                    "gt" | "go_to" => {
                        let arg = Self::parse_next_as_u16(&mut words, 1);
                        if arg.is_some() {
                            self.runto_address = arg;
                            println!("Going to {:#x}", arg.unwrap());
                            CommandResult::ResumeGame
                        } else {
                            CommandResult::None
                        }
                    }
                    "s" | "status" => {
                        println!("{:?}", mem.r_i());
                        CommandResult::None
                    }
                    "gs" | "graphics_status" => {
                        println!(
                            "LCDC: {:#x}, STAT: {:#x}, LY: {:#x}",
                            mem.read_8_sys(ADDRESS_LCDC),
                            mem.read_8_sys(ADDRESS_STAT),
                            mem.read_8_sys(ADDRESS_LY)
                        );
                        CommandResult::None
                    }
                    "n" | "next" => {
                        println!("Running next instruction");
                        self.pause_next = true;
                        CommandResult::ResumeGame
                    }
                    "ns" | "next_status" => {
                        println!("Running next instruction and printing status");
                        self.pause_next = true;
                        self.stat_next = true;
                        CommandResult::ResumeGame
                    }
                    "r" | "read" => {
                        match Self::parse_next_as_u16(&mut words, 1) {
                            Some(addr) => {
                                println!("Value of {addr:#x} is {:#x}", mem.read_8_sys(addr));
                            }
                            None => {}
                        };
                        CommandResult::None
                    }
                    "b" | "break" => {
                        match Self::parse_next_as_u16(&mut words, 1) {
                            Some(addr) => {
                                self.break_pc_addrs.push(addr);
                                println!("Will break when {addr:#x} is in pc");
                            }
                            None => {}
                        };
                        CommandResult::None
                    }
                    "bc" | "break_clear" => {
                        self.break_pc_addrs.clear();
                        println!("Breakpoints cleared");
                        CommandResult::None
                    }
                    "w" | "watch" => {
                        match Self::parse_next_as_u16(&mut words, 1) {
                            Some(addr) => {
                                self.watch_addrs.push(addr);
                                println!("Value of {addr:#x} is {:#x}", mem.read_8_sys(addr));
                            }
                            None => {}
                        };
                        CommandResult::None
                    }
                    "wc" | "watch_clear" => {
                        self.watch_addrs.clear();
                        println!("Watches cleared");
                        CommandResult::None
                    }
                    "q" | "quit" => {
                        panic!("Quit command issued");
                    }
                    "j" | "jumps" => {
                        if DEBUG_TRACK_JUMPS {
                            let mut highlight_addrs = Vec::new();
                            loop {
                                if words.clone().count() == 0 {
                                    break;
                                }

                                match Self::parse_next_as_u16(&mut words, highlight_addrs.len() + 1) {
                                    Some(addr) => {
                                        highlight_addrs.push(addr);
                                    }
                                    None => { break; }
                                };
                            }

                            metrics.print_jumps(&highlight_addrs);
                        } else {
                            println!("Jumps not being tracked");
                        }
                        CommandResult::None
                    }
                    "jc" | "jumps_clear" => {
                        if DEBUG_TRACK_JUMPS {
                            metrics.clear_jumps();
                            println!("jumps cleared");
                        } else {
                            println!("Jumps not being tracked");
                        }
                        CommandResult::None
                    }
                    "h" | "help" => {
                        println!(indoc! {"
                            Note: Addresses should be specified in hexadecimal without any prefix. eg 0xbeef is beef
                            Commands (all have a shorthand of the first characters of each word):
                            pause
                            go
                            go_to <addr>
                            status
                            graphic_sstatus
                            next
                            next_status - runs the next instruction, breaks, and prints status
                            read <addr>
                            break <addr> - program will break at this pc value
                            break_clear
                            watch <addr> - prints this address's value when the program breaks/pauses
                            watch_clear
                            jumps
                            jumps_clear
                            quit
                            help
                        "});
                        CommandResult::None
                    }
                    _ => {
                        println!("Unknown command received. Type 'help' for help.");
                        CommandResult::None
                    }
                }
            }
            Err(_) => CommandResult::None,
        }
    }

    fn parse_next_as_u16(split: &mut dyn Iterator<Item = &str>, argument_number: usize) -> Option<u16> {
        match split.next() {
            Some(val_str) => match u16::from_str_radix(val_str, 16) {
                Ok(val) => Some(val),
                Err(e) => {
                    println!("Failed to read argument {argument_number} as u16: {e}");
                    None
                }
            },
            None => {
                println!("Expected a u16 as argument {argument_number} but found nothing");
                None
            }
        }
    }

    fn print_watches(&self, mem: &dyn MemoryController) {
        for addr in &self.watch_addrs {
            println!("Value of {addr:#x} is {:#x}", mem.read_8_sys(*addr));
        }
    }

    // From https://stackoverflow.com/a/55201400
    fn spawn_stdin_channel() -> Receiver<String> {
        let (tx, rx) = mpsc::channel::<String>();
        thread::spawn(move || loop {
            let mut buffer = String::new();
            io::stdin().read_line(&mut buffer).unwrap();
            tx.send(buffer).unwrap();
        });
        rx
    }
}
