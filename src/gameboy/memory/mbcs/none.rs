use crate::gameboy::memory::mbcs::MBC;
use crate::gameboy::memory::rom::Rom;

pub struct MBCNone {
    pub rom: Rom
}

impl MBC for MBCNone {
    fn read(&self, address: u16) -> u8 {
        self.rom.read(address)
    }

    fn write(&mut self, _: u16, _: u8) {
        // No MBC ignores writes
    }

    fn ram_read(&self, _: u16) -> u8 {
        // Unused Gameboy RAM reads as 0xFF
        0xFF
    }

    fn ram_write(&mut self, _: u16, _: u8) {
        // We don't have RAM
    }

    fn step (&mut self) {
        // We don't need to do anything here
    }
}

impl MBCNone {
    pub fn new(rom: Rom) -> Self {
        MBCNone {
            rom
        }
    }
}
