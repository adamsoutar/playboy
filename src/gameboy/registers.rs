use crate::gameboy::helpers::*;
use crate::gameboy::memory::memory::Memory;
use crate::gameboy::interrupts::*;
use crate::gameboy::gpu::Gpu;
use alloc::string::String;
use alloc::format;

pub struct Registers {
    pub a: u8,
    pub b: u8,
    pub c: u8,
    pub d: u8,
    pub e: u8,
    pub f: u8,
    pub h: u8,
    pub l: u8,

    pub sp: u16,
    pub pc: u16
}
impl Registers {
    fn set_flag(&mut self, flag_index: u8, bit: u8) {
        set_bit(&mut self.f, 4 + flag_index, bit)
    }
    pub fn set_carry_flag (&mut self, bit: u8) {
        self.set_flag(0, bit)
    }
    pub fn set_half_carry_flag (&mut self, bit: u8) {
        self.set_flag(1, bit)
    }
    pub fn set_operation_flag (&mut self, bit: u8) {
        self.set_flag(2, bit)
    }
    pub fn set_zero_flag (&mut self, bit: u8) {
        self.set_flag(3, bit)
    }

    pub fn set_flags(&mut self, zero: u8, operation: u8, half_carry: u8, carry: u8) {
        self.set_carry_flag(carry);
        self.set_half_carry_flag(half_carry);
        self.set_operation_flag(operation);
        self.set_zero_flag(zero);
    }

    fn get_flag(&self, flag_index: u8) -> u8 {
        (self.f >> (4 + flag_index)) & 0x1
    }
    pub fn get_carry_flag (&self) -> u8 {
        self.get_flag(0)
    }
    pub fn get_half_carry_flag (&self) -> u8 {
        self.get_flag(1)
    }
    pub fn get_operation_flag (&self) -> u8 {
        self.get_flag(2)
    }
    pub fn get_zero_flag (&self) -> u8 {
        self.get_flag(3)
    }

    pub fn get_af (&self) -> u16 {
        combine_u8(self.a, self.f)
    }
    pub fn set_af (&mut self, value: u16) {
        let (b1, b2) = split_u16(value);
        self.a = b2; self.f = b1 & 0xF0;
    }

    pub fn get_bc (&self) -> u16 {
        combine_u8(self.b, self.c)
    }
    pub fn set_bc (&mut self, value: u16) {
        let (b1, b2) = split_u16(value);
        self.b = b2; self.c = b1;
    }

    pub fn get_de (&self) -> u16 {
        combine_u8(self.d, self.e)
    }
    pub fn set_de (&mut self, value: u16) {
        let (b1, b2) = split_u16(value);
        self.d = b2; self.e = b1;
    }

    pub fn get_hl (&self) -> u16 {
        combine_u8(self.h, self.l)
    }
    pub fn set_hl (&mut self, value: u16) {
        let (b1, b2) = split_u16(value);
        self.h = b2; self.l = b1;
    }

    // These are left to right from the "GoldenCrystal Gameboy Z80 CPU Opcodes" PDF
    // The "sp" flag indicates whether 0b11 refers to the SP or AF
    fn set_combined_register_base (&mut self, register: u8, value: u16, sp: bool) {
        match register {
            0b00 => self.set_bc(value),
            0b01 => self.set_de(value),
            0b10 => self.set_hl(value),
            0b11 => if sp { self.sp = value } else { self.set_af(value) },
            _ => panic!("Invalid combined register set")
        }
    }
    fn get_combined_register_base (&self, register: u8, sp: bool) -> u16 {
        match register {
            0b00 => self.get_bc(),
            0b01 => self.get_de(),
            0b10 => self.get_hl(),
            0b11 => if sp { self.sp } else { self.get_af() },
            _ => panic!("Invalid combined register get")
        }
    }

    pub fn get_combined_register (&self, register: u8) -> u16 {
        self.get_combined_register_base(register, true)
    }
    pub fn set_combined_register(&mut self, register: u8, value: u16) {
        self.set_combined_register_base(register, value, true)
    }
    pub fn get_combined_register_alt (&self, register: u8) -> u16 {
        self.get_combined_register_base(register, false)
    }
    pub fn set_combined_register_alt (&mut self, register: u8, value: u16) {
        self.set_combined_register_base(register, value, false)
    }


    pub fn set_singular_register (&mut self, register: u8, value: u8, mem: &mut Memory, ints: &mut Interrupts, gpu: &mut Gpu) {
        match register {
            0b000 => self.b = value,
            0b001 => self.c = value,
            0b010 => self.d = value,
            0b011 => self.e = value,
            0b100 => self.h = value,
            0b101 => self.l = value,
            0b110 => mem.write(ints, gpu, self.get_hl(), value),
            0b111 => self.a = value,
            _ => panic!("Invalid singular register set")
        }
    }

    pub fn get_singular_register (&self, register: u8, mem: &Memory, ints: &Interrupts, gpu: &Gpu) -> u8 {
        match register {
            0b000 => self.b,
            0b001 => self.c,
            0b010 => self.d,
            0b011 => self.e,
            0b100 => self.h,
            0b101 => self.l,
            0b110 => mem.read(ints, gpu, self.get_hl()),
            0b111 => self.a,
            _ => panic!("Invalid singular register get")
        }
    }

    pub fn debug_dump (&self) -> String {
        format!("AF: {:#06x} | BC: {:#06x} | DE: {:#06x} | HL: {:#06x}", self.get_af(), self.get_bc(), self.get_de(), self.get_hl())
    }

    pub fn new () -> Registers {
        // NOTE: These values are what's in the registers after the boot rom,
        //       since we don't run that.
        Registers {
            a: 0x01, b: 0x00, c: 0x13, d: 0x00,
            e: 0xD8, f: 0xB0, h: 0x01, l: 0x4D,
            sp: 0xFFFE, pc: 0x100
        }
    }
}
