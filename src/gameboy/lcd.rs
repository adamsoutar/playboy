use crate::gameboy::interrupts::{Interrupts, InterruptReason};

#[derive(Clone, Copy)]
pub enum GreyShade {
    White,
    LightGrey,
    DarkGrey,
    Black
}
impl From<u8> for GreyShade {
    fn from(n: u8) -> GreyShade {
        match n {
            0 => GreyShade::White,
            1 => GreyShade::LightGrey,
            2 => GreyShade::DarkGrey,
            3 => GreyShade::Black,
            _ => panic!("Invalid grey shade id {}", n)
        }
    }
}

#[derive(Clone, Copy)]
pub struct LcdControl {
    pub display_enable: bool,
    pub window_tile_map_display_select: bool,
    pub window_enable: bool,
    pub bg_and_window_data_select: bool,
    pub bg_tile_map_display_select: bool,
    pub obj_size: bool,
    pub obj_enable: bool,
    pub bg_display: bool
}
impl LcdControl {
    pub fn new () -> LcdControl {
        // TODO: Check
        // PAC-MAN doesn't boot unless LCDC is initialised
        // with display_enable on, but
        // https://github.com/mohanson/gameboy/blob/master/src/gpu.rs#L100
        // seems to init without it, and manages to boot PAC-MAN
        LcdControl::from(0b11001000)
    }
}
impl From<u8> for LcdControl {
    fn from(n: u8) -> LcdControl {
        LcdControl {
            bg_display: (n & 0b0000_0001) == 0b0000_0001,
            obj_enable: (n & 0b0000_0010) == 0b0000_0010,
            obj_size: (n & 0b0000_0100) == 0b0000_0100,
            bg_tile_map_display_select: (n & 0b0000_1000) == 0b0000_1000,
            bg_and_window_data_select: (n & 0b0001_0000) == 0b0001_0000,
            window_enable: (n & 0b0010_0000) == 0b0010_0000,
            window_tile_map_display_select: (n & 0b0100_0000) == 0b0100_0000,
            display_enable: (n & 0b1000_0000) == 0b1000_0000
        }
    }
}
impl From<LcdControl> for u8 {
    fn from (lcd: LcdControl) -> u8 {
        lcd.bg_display as u8 |
        (lcd.obj_enable as u8) << 1 |
        (lcd.obj_size as u8) << 2 |
        (lcd.bg_tile_map_display_select as u8) << 3 |
        (lcd.bg_and_window_data_select as u8) << 4 |
        (lcd.window_enable as u8) << 5 |
        (lcd.window_tile_map_display_select as u8) << 6 |
        (lcd.display_enable as u8) << 7
    }
}

#[derive(PartialEq, Clone)]
pub enum LcdMode {
    HBlank = 0,
    VBlank = 1,
    OAMSearch = 2,
    Transfer = 3
}

#[derive(Clone, Copy)]
pub struct LcdStatus {
    pub lyc: bool,
    pub oam_interrupt: bool,
    pub vblank_interrupt: bool,
    pub hblank_interrupt: bool,
    pub coincidence_flag: bool,
    pub mode_flag: u8
}
impl LcdStatus {
    pub fn get_mode (&self) -> LcdMode {
        match self.mode_flag {
            0 => LcdMode::HBlank,
            1 => LcdMode::VBlank,
            2 => LcdMode::OAMSearch,
            3 => LcdMode::Transfer,
            _ => panic!("Invalid LCD mode")
        }
    }

    pub fn set_data (&mut self, data: u8, ints: &mut Interrupts) {
        // There's actually a DMG GPU bug when writing to LCDStat
        // where sometimes it fires off an interrupt at the wrong time
        // https://robertovaccari.com/blog/2020_09_26_gameboy/
        match self.get_mode() {
            LcdMode::HBlank | LcdMode::VBlank => {
                if self.lyc {
                    ints.raise_interrupt(InterruptReason::LCDStat)
                }
            },
            _ => {}
        }

        let new_stat = LcdStatus::from(data);
        self.lyc = new_stat.lyc;
        self.oam_interrupt = new_stat.oam_interrupt;
        self.vblank_interrupt = new_stat.vblank_interrupt;
        self.hblank_interrupt = new_stat.hblank_interrupt;
        // NOTE: We *don't* set the coincidence_flag or mode_flag,
        //       they're read only
    }

    pub fn set_mode (&mut self, mode: LcdMode) {
        self.mode_flag = mode as u8;
    }

    pub fn new () -> LcdStatus {
        // LCD starts in OAMSearch, not HBlank
        LcdStatus::from(0b000000_10)
    }
}
impl From<u8> for LcdStatus {
    fn from(n: u8) -> LcdStatus {
        LcdStatus {
            lyc: (n & 0b1000000) == 0b1000000,
            oam_interrupt: (n & 0b100000) == 0b100000,
            vblank_interrupt: (n & 0b10000) == 0b10000,
            hblank_interrupt: (n & 0b1000) == 0b1000,
            coincidence_flag: (n & 0b100) == 0b100,
            mode_flag: n & 0b11
        }
    }
}
impl From<LcdStatus> for u8 {
    fn from(lcd: LcdStatus) -> u8 {
        lcd.mode_flag |
        (lcd.coincidence_flag as u8) << 2 |
        (lcd.hblank_interrupt as u8) << 3 |
        (lcd.vblank_interrupt as u8) << 4 |
        (lcd.oam_interrupt as u8) << 5 |
        (lcd.lyc as u8) << 6
    }
}
