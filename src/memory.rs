use std::fmt::Debug;

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

fn u8s_to_u16(h: u8, l: u8) -> u16 {
    ((h as u16) << 8) | l as u16
}

fn u16_to_u8s(d: u16) -> (u8, u8) {
    ((d >> 8) as u8, d as u8)
}

#[repr(C)]
#[derive(Debug, Default)]
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

#[repr(C)]
#[derive(Default, Debug)]
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

pub trait MemoryController {
    fn read_8(&self, addr: u16) -> u8;
    fn read_8_sys(&self, addr: u16) -> u8;
    fn write_8(&mut self, addr: u16, val: u8);
    fn write_8_sys(&mut self, addr: u16, val: u8);
    // remove?
    fn mut_8(&mut self, addr: u16) -> &mut u8;
    fn r(&mut self) -> &mut Registers;
    fn r_i(&self) -> &Registers;
    fn ime(&mut self) -> &mut bool;
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
