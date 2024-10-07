use std::fmt::{format, Debug};

use bitflags::bitflags;

use crate::constants::*;

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

fn u8s_to_u16(h: u8, l: u8) -> u16 {
    ((h as u16) << 8) | l as u16
}

fn u16_to_u8s(d: u16) -> (u8, u8) {
    ((d >> 8) as u8, d as u8)
}

#[repr(C)]
#[derive(Default)]
pub struct RegisterPair {
    pub ind: (u8, u8),
}

impl RegisterPair {
    pub fn r16(&self) -> u16 {
        u8s_to_u16(self.ind.0, self.ind.1)
    }

    pub fn s16(&mut self, val: u16) {
        let vals = u16_to_u8s(val);
        self.ind = vals;
    }

    pub fn uinc16(&mut self) {
        let result = self.ind.1.overflowing_add(1);
        self.ind.1 = result.0;
        if result.1 {
            self.ind.0 = self.ind.0.wrapping_add(1);
        }
    }

    pub fn udec16(&mut self) {
        let result = self.ind.1.overflowing_sub(1);
        self.ind.1 = result.0;
        if result.1 {
            self.ind.0 = self.ind.0.wrapping_sub(1);
        }
    }
}

impl Debug for RegisterPair {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RegisterPair")
            .field("ind", &format!("({:#04x}, {:#04x})", self.ind.0, self.ind.1))
            .finish()
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

impl Registers {
    pub fn set_flags_unchecked(&mut self, data: u8) {
        unsafe {
            self.f = RegisterFlags::from_bits_unchecked(data);
        }
    }
}

impl Debug for Registers {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Registers")
            .field("a", &format!("{:#04x}", self.a))
            .field("f", &self.f)
            .field("bc", &self.bc)
            .field("de", &self.de)
            .field("hl", &self.hl)
            .field("pc", &format!("{:#06x}", self.pc))
            .field("sp", &format!("{:#06x}", self.sp))
            .finish()
    }
}

#[derive(Default)]
pub struct Inputs {
    pub down: bool,
    pub up: bool,
    pub left: bool,
    pub right: bool,
    pub start: bool,
    pub select: bool,
    pub b: bool,
    pub a: bool,
    pub reset: bool,
}

#[derive(Default)]
pub struct MemorySharedData {
    pub r: Registers,
    pub ime: bool,
    // todo: CPU and PPU access to memory is restricted while a DMA transfer is active
    pub dma_source_address: u16,
    pub inputs: Inputs
}

pub trait MemoryController {
    fn shared_data(&self) -> &MemorySharedData;
    fn shared_data_mut(&mut self) -> &mut MemorySharedData;
    fn read_8(&self, addr: u16) -> u8;
    fn read_8_sys(&self, addr: u16) -> u8;

    fn write_8(&mut self, addr: u16, mut val: u8) {
        match addr {
            ADDRESS_JOYP => {
                let joyp_orig = self.read_8_sys(ADDRESS_JOYP);
                self.write_8_sys(ADDRESS_JOYP, (joyp_orig & !0x30) | (val & 0x30));
                self.process_input();
                return;
            },
            ADDRESS_DIV => {
                val = 0;
            },
            ADDRESS_STAT => {
                let stat = self.read_8_sys(ADDRESS_STAT);
                // Bits 0, 1, and 2 are read-only for the CPU
                val = (val & !7) | (stat & 7);
            },
            ADDRESS_LY => {
                // LY is read-only
                return;
            },
            ADDRESS_DMA_CONTROL => {
                self.shared_data_mut().dma_source_address = val as u16 * 0x100;
            },
            _ => {},
        }

        self.write_8_sys(addr, val);
    }
    
    fn write_8_sys(&mut self, addr: u16, val: u8);
    // remove?
    fn mut_8(&mut self, addr: u16) -> &mut u8;
    fn r(&mut self) -> &mut Registers {
        &mut self.shared_data_mut().r
    }
    fn r_i(&self) -> &Registers {
        &self.shared_data().r
    }
    fn ime(&mut self) -> &mut bool {
        &mut self.shared_data_mut().ime
    }

    fn process_input(&mut self) {
        let joyp_orig = self.read_8_sys(ADDRESS_JOYP);
        let input = &self.shared_data().inputs;

        let mut joyp_new = joyp_orig | 0x0f;
        if (joyp_new & 0x30) != 0x30 {
            if joyp_new & (1 << 4) != 0 {
                if input.right {
                    joyp_new &= !(1);
                }
    
                if input.left {
                    joyp_new &= !(1 << 1);
                }
    
                if input.up {
                    joyp_new &= !(1 << 2);
                }
    
                if input.down {
                    joyp_new &= !(1 << 3);
                }
            }
            
            if joyp_new & (1 << 5) != 0 {
                if input.a {
                    joyp_new &= !(1);
                }
    
                if input.b {
                    joyp_new &= !(1 << 1);
                }
    
                if input.select {
                    joyp_new &= !(1 << 2);
                }
    
                if input.start {
                    joyp_new &= !(1 << 3);
                }
            }
        }

        self.write_8_sys(ADDRESS_JOYP, joyp_new);

        let interrupt_request = self.read_8_sys(ADDRESS_IF);
        if (joyp_new & 0x0f) != 0x0f && joyp_new != joyp_orig {
            // it seems like IF is not cleared if the button is no longer held. todo figure out?
            self.write_8_sys(ADDRESS_IF, interrupt_request | (1 << 4));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::RegisterPair;

    #[test]
    fn register_pair_uinc_16_1() {
        let mut r = RegisterPair::default();
        r.s16(0);
        r.uinc16();

        assert_eq!(1, r.r16())
    }

    #[test]
    fn register_pair_uinc_16_lower_overflow() {
        let mut r = RegisterPair::default();
        r.s16(0x00FF);
        r.uinc16();

        assert_eq!(0x0100, r.r16())
    }

    #[test]
    fn register_pair_uinc_16_total_overflow() {
        let mut r = RegisterPair::default();
        r.s16(0xFFFF);
        r.uinc16();

        assert_eq!(0, r.r16())
    }

    #[test]
    fn register_pair_udec_16_1() {
        let mut r = RegisterPair::default();
        r.s16(1);
        r.udec16();

        assert_eq!(0, r.r16())
    }

    #[test]
    fn register_pair_udec_16_half_underflow() {
        let mut r = RegisterPair::default();
        r.s16(0x0100);
        r.udec16();

        assert_eq!(0x00FF, r.r16())
    }

    #[test]
    fn register_pair_udec_16_total_underflow() {
        let mut r = RegisterPair::default();
        r.s16(0);
        r.udec16();

        assert_eq!(0xFFFF, r.r16())
    }
}
