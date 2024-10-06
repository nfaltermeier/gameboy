use crate::{
    constants::*,
    memory::{MemoryController, MemorySharedData},
};

#[repr(C)]
pub struct BasicMemory {
    pub shared_data: MemorySharedData,
    rom: Vec<u8>,       // 0x0000 - 0x7FFF
    vram: [u8; 0x2000], // 0x8000 - 0x9FFF
    ram: [u8; 0x2000],  // 0xC000 - 0xDFFF
    oam: [u8; 0xA0],
    system_mem: [u8; 0x100],
}

impl BasicMemory {
    pub fn new(rom: Vec<u8>) -> Self {
        Self {
            shared_data: Default::default(),
            rom,
            vram: [0; 0x2000],
            ram: [0; 0x2000],
            oam: [0; 0xA0],
            system_mem: [0; 0x100],
        }
    }
}

impl MemoryController for BasicMemory {
    fn shared_data(&self) -> &MemorySharedData {
        &self.shared_data
    }

    fn shared_data_mut(&mut self) -> &mut MemorySharedData {
        &mut self.shared_data
    }

    fn read_8(&self, addr: u16) -> u8 {
        self.read_8_sys(addr)
    }

    fn read_8_sys(&self, addr: u16) -> u8 {
        if addr < 0x8000 {
            self.rom[addr as usize]
        } else if addr < 0xA000 {
            self.vram[(addr - 0x8000) as usize]
        } else if addr < 0xC000 {
            panic!(
                "Tried to read cartridge RAM at {:#x} but cartridge has no RAM",
                addr
            )
        } else if addr < 0xE000 {
            self.ram[(addr - 0xC000) as usize]
        } else if addr < 0xFE00 {
            // Nintendo prohibits use but hardware functionality is documented as echoing C000
            self.read_8(addr - 0x2000)
        } else if addr < 0xFEA0 {
            self.oam[(addr - 0xFE00) as usize]
        } else if addr < 0xFF00 {
            todo!(
                "Tried to read prohibited space at {:#x}. Hardware behavior not implemented yet.",
                addr
            )
        } else {
            self.system_mem[(addr - 0xFF00) as usize]
        }
    }

    fn write_8_sys(&mut self, addr: u16, val: u8) {
        if addr < 0x8000 {
            // writing to ROM is skipped
        } else if addr < 0xA000 {
            if crate::debug::flags::DEBUG_PRINT_VRAM_WRITES {
                println!("Writing {:#b} to VRAM {:#x}", val, addr);
            }
            self.vram[(addr - 0x8000) as usize] = val;
        } else if addr < 0xC000 {
            panic!(
                "Tried to write cartridge RAM at {:#x} but cartridge has no RAM",
                addr
            )
        } else if addr < 0xE000 {
            self.ram[(addr - 0xC000) as usize] = val;
        } else if addr < 0xFE00 {
            // Nintendo prohibits use but hardware functionality is documented as echoing C000
            self.write_8(addr - 0x2000, val);
        } else if addr < 0xFEA0 {
            self.oam[(addr - 0xFE00) as usize] = val;
        } else if addr < 0xFF00 {
            // todo!("Tried to write prohibited space at {:#x}. Hardware behavior not implemented yet.", addr)
        } else {
            self.system_mem[(addr - 0xFF00) as usize] = val;
        }
    }

    fn mut_8(&mut self, addr: u16) -> &mut u8 {
        if addr < 0x8000 {
            panic!("Tried to get mutable ref to {:#x}", addr)
        } else if addr < 0xA000 {
            &mut self.vram[(addr - 0x8000) as usize]
        } else if addr < 0xC000 {
            panic!(
                "Tried to get mutable ref to cartridge RAM at {:#x} but cartridge has no RAM",
                addr
            )
        } else if addr < 0xE000 {
            &mut self.ram[(addr - 0xC000) as usize]
        } else if addr < 0xFE00 {
            // Nintendo prohibits use but hardware functionality is documented as echoing C000
            self.mut_8(addr - 0x2000)
        } else if addr < 0xFEA0 {
            &mut self.oam[(addr - 0xFE00) as usize]
        } else if addr < 0xFF00 {
            todo!("Tried to get mutable ref to prohibited space at {:#x}. Hardware behavior not implemented yet.", addr)
        } else {
            &mut self.system_mem[(addr - 0xFF00) as usize]
        }
    }
}

impl Default for BasicMemory {
    fn default() -> Self {
        Self {
            shared_data: Default::default(),
            rom: vec![0; 0x8000],
            vram: [0; 0x2000],
            ram: [0; 0x2000],
            oam: [0; 0xA0],
            system_mem: [0; 0x100],
        }
    }
}
