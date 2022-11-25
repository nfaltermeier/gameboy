use bitflags::bitflags;

bitflags! {
    #[repr(C)]
    #[derive(Default)]
    pub struct RegisterFlags: u8 {
        const Z  = 0b10000000;
        const N  = 0b01000000;
        const H  = 0b00100000;
        const CY = 0b00010000;
    }
}

#[repr(C)]
pub union RegisterPair {
    pub comb: u16,
    pub ind: (u8, u8),
}

impl Default for RegisterPair {
    fn default() -> Self {
        RegisterPair { comb: 0 }
    }
}

#[repr(C)]
#[derive(Default)]
pub struct Registers {
    pub a: u8,
    pub f: RegisterFlags,
    pub bc: RegisterPair,
    pub de: RegisterPair,
    pub hl: RegisterPair,
    pub pc: u16,
    pub sp: u16,
}

#[repr(C)]
pub struct Memory {
    pub r: Registers,
    mem: [u8; 0x10000],
}

impl Memory {
    pub fn read_8(&self, addr: u16) -> u8 {
        self.mem[addr as usize]
    }

    pub fn write_8(&mut self, addr: u16, val: u8) {
        self.mem[addr as usize] = val;
    }

    pub fn mut_8(&mut self, addr: u16) -> &mut u8 {
        &mut self.mem[addr as usize]
    }
}

impl Default for Memory {
    fn default() -> Self {
        Self {
            r: Default::default(),
            mem: [0; 0x10000],
        }
    }
}
