// "WRAM" is Work RAM, not Wave RAM
pub const WRAM_SIZE: usize = 8192;
pub const VRAM_SIZE: usize = 8192;
pub const HRAM_SIZE: usize = 127;
pub const OAM_SIZE: usize = 160;
pub const WAVE_RAM_SIZE: usize = 16;

// Excluding invisible areas such as those above and to
// the left of the screen
pub const SCREEN_WIDTH: usize = 160;
pub const SCREEN_HEIGHT: usize = 144;

pub const SCREEN_BUFFER_SIZE: usize = SCREEN_WIDTH * SCREEN_HEIGHT;
pub const SCREEN_RGBA_SLICE_SIZE: usize = SCREEN_BUFFER_SIZE * 4;

pub const CLOCK_SPEED: usize = 4194304;
pub const FRAME_RATE: usize = 60;
pub const CYCLES_PER_FRAME: usize = CLOCK_SPEED / FRAME_RATE;

// MBC_ROM_START is 0
pub const MBC_ROM_END: u16 = 0x7FFF;

pub const MBC_RAM_START: u16 = 0xA000;
pub const MBC_RAM_END: u16 = 0xBFFF;

pub const VRAM_START: u16 = 0x8000;
pub const VRAM_END: u16 = 0x9FFF;

pub const WRAM_START: u16 = 0xC000;
pub const WRAM_END: u16 = 0xDFFF;

pub const ECHO_RAM_START: u16 = 0xE000;
pub const ECHO_RAM_END: u16 = 0xFDFF;

pub const OAM_START: u16 = 0xFE00;
pub const OAM_END: u16 = 0xFE9F;

pub const UNUSABLE_MEMORY_START: u16 = 0xFEA0;
pub const UNUSABLE_MEMORY_END: u16 = 0xFEFF;

pub const LINK_CABLE_SB: u16 = 0xFF01;
pub const LINK_CABLE_SC: u16 = 0xFF02;

pub const APU_START: u16 = 0xFF10;
pub const APU_END: u16 = 0xFF3F;

pub const WAVE_RAM_START: u16 = 0xFF30;
pub const WAVE_RAM_END: u16 = 0xFF3F;

pub const HRAM_START: u16 = 0xFF80;
pub const HRAM_END: u16 = 0xFFFE;

// This isn't *strictly* true, but it only overlaps CGB
// functionality, so it's OK.
pub const LCD_DATA_START: u16 = 0xFF40;
pub const LCD_DATA_END: u16 = 0xFF4F;

pub const INTERRUPT_ENABLE_ADDRESS: u16 = 0xFFFF;
pub const INTERRUPT_FLAG_ADDRESS: u16 = 0xFF0F;

pub mod gpu_timing {
    // Total line size incl. HBlank
    pub const HTOTAL: u16 = 456;

    // lx coordinate where Transfer begins
    pub const HTRANSFER_ON: u16 = 80;

    // Start of HBlank
    pub const HBLANK_ON: u16 = 252;

    // Total vertical lines incl. VBlank
    pub const VTOTAL:   u8 = 154;
    // Start of VBlank
    pub const VBLANK_ON: u8 = 144;

    // Number of CPU cycles it takes to do a DMA
    pub const DMA_CYCLES: u8 = 160;
}
