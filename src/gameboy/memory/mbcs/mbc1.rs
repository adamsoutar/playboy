use crate::gameboy::memory::mbcs::MBC;
use crate::gameboy::memory::rom::Rom;
use crate::gameboy::memory::battery_backed_ram::BatteryBackedRam;
use crate::gameboy::cartridge::Cartridge;

// 16KB (one bank size) in bytes
pub const KB_16: usize = 16_384;

pub struct MBC1 {
    pub rom: Rom,
    pub rom_bank: u8,

    pub ram: BatteryBackedRam,
    pub ram_enabled: bool,

    has_shown_ram_warning: bool
}

impl MBC for MBC1 {
    fn read(&self, address: u16) -> u8 {
        match address {
            0x0 ..= 0x3FFF => self.read_bank(0, address),
            0x4000 ..= 0x7FFF => self.read_bank(self.rom_bank, address - 0x4000),
            _ => panic!("Unsupported MBC1 read at {:#06x}", address)
        }
    }

    fn write(&mut self, address: u16, value: u8) {
        match address {
            0x0000 ..= 0x1FFF => {
                self.ram_enabled = (value & 0x0A) == 0x0A;
            },
            0x2000 ..= 0x3FFF => {
                // TODO: Bank numbers are masked to the max bank number
                // TODO: Upper/RAM banking support
                let mut n = value & 0b11111;
                if n == 0 { n = 1 }
                self.rom_bank = n
            },
            // 0x4000 ..= 0x5FFF => {
            //     panic!("Unsupported upper bank number or RAM banking in MBC1")
            // },
            // 0x6000 ..= 0x7FFF => {
            //     panic!("Unsupported MBC1 mode select write")
            // },
            _ => {}//panic!("Unsupported MBC1 write at {:#06x} (value: {:#04x})", address, value)
        }
    }

    fn ram_read(&self, address: u16) -> u8 {
        if !self.ram_enabled && !self.has_shown_ram_warning {
            // println!("[WARN] MBC1 RAM read while disabled");
        }

        // When an address outside of RAM space is read, the gameboy
        // doesn't seem to be intended to crash.
        // Not sure what to return here, but unusable RAM on the GB itself
        // returns 0xFF
        if address as usize >= self.ram.size { return 0xFF }

        self.ram.read(address)
    }

    fn ram_write(&mut self, address: u16, value: u8) {
        if !self.ram_enabled && !self.has_shown_ram_warning { 
            // println!("[WARN] MBC1 RAM write while disabled");
            // Otherwise the game is slowed down by constant debug printing
            self.has_shown_ram_warning = true;
        }

        // See note in ram_read
        if address as usize >= self.ram.size { return }

        self.ram.write(address, value)
    }

    fn step(&mut self) {
        self.ram.step()
    }
}

impl MBC1 {
    fn read_bank(&self, bank: u8, address: u16) -> u8 {
        let ub = bank as usize;
        let ua = address as usize;
        self.rom.bytes[KB_16 * ub + ua]
    }

    pub fn new (cart_info: Cartridge, rom: Rom) -> Self {
        // TODO: Banked RAM
        if cart_info.ram_size > 8_192 { 
            panic!("gbrs doesn't support banked (>=32K) MBC1 RAM");
        }

        let has_battery = cart_info.cart_type == 0x03;
        MBC1 {
            rom,
            ram_enabled: false,
            rom_bank: 1,
            ram: BatteryBackedRam::new(
                cart_info.ram_size, 
                has_battery, 
                &cart_info.rom_path[..]
            ),
            has_shown_ram_warning: false
        }
    }
}
