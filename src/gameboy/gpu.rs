use crate::gameboy::constants::*;
use crate::gameboy::lcd::*;
use crate::gameboy::memory::ram::Ram;
use crate::gameboy::memory::memory::Memory;
use crate::gameboy::interrupts::*;
use alloc::vec::Vec;
use alloc::vec;

#[derive(Clone)]
pub struct Sprite {
    pub y_pos: i32,
    pub x_pos: i32,
    pub pattern_id: u8,

    pub above_bg: bool,
    pub y_flip: bool,
    pub x_flip: bool,
    pub use_palette_0: bool
}

pub struct Gpu {
    // This is the WIP frame that the GPU draws to
    frame: [GreyShade; SCREEN_BUFFER_SIZE],
    // This is the frame read by the GUI,
    // it's only updated in VBlank
    finished_frame: [GreyShade; SCREEN_BUFFER_SIZE],

    // X and Y of background position
    scy: u8,
    scx: u8,

    // X and Y of the Window
    wy: u8,
    wx: u8,

    // The scan-line Y co-ordinate
    ly: u8,
    // If ly is lyc ("compare") and the interrupt is enabled,
    // an LCD Status interrupt is flagged
    lyc: u8,

    // Scan-line X co-ordinate
    // This isn't a real readable Gameboy address, it's just for internal tracking
    lx: u16,

    bg_pallette: u8,
    sprite_pallete_1: u8,
    sprite_pallete_2: u8,

    status: LcdStatus,
    control: LcdControl,

    // "Object Attribute Memory" - Sprite properties
    oam: Ram,

    dma_source: u8,
    dma_cycles: u8,

    sprite_cache: Vec<Sprite>
}

impl Gpu {
    pub fn raw_write (&mut self, raw_address: u16, value: u8, ints: &mut Interrupts) {
        match raw_address {
            OAM_START ..= OAM_END => self.oam.write(raw_address - OAM_START, value),

            0xFF40 => self.control = LcdControl::from(value),
            0xFF41 => self.status.set_data(value, ints),
            0xFF42 => self.scy = value,
            0xFF43 => self.scx = value,
            0xFF45 => self.lyc = value,

            0xFF46 => self.begin_dma(value),

            0xFF4A => self.wy = value,
            0xFF4B => self.wx = value,

            0xFF47 => self.bg_pallette = value,
            0xFF48 => self.sprite_pallete_1 = value,
            0xFF49 => self.sprite_pallete_2 = value,

            // CGB only
            0xFF4F => {},

            // The Y Scanline is read only.
            // Space Invaders writes here. As a bug?
            0xFF44 => {},

            _ => panic!("Unsupported GPU write at {:#06x}", raw_address)
        }
    }
    pub fn raw_read (&self, raw_address: u16) -> u8 {
        match raw_address {
            OAM_START ..= OAM_END => self.oam.read(raw_address - OAM_START),

            0xFF40 => u8::from(self.control),
            0xFF41 => u8::from(self.status),
            0xFF42 => self.scy,
            0xFF43 => self.scx,
            0xFF44 => self.ly,
            0xFF45 => self.lyc,

            0xFF46 => self.dma_source,

            0xFF4A => self.wy,
            0xFF4B => self.wx,

            0xFF47 => self.bg_pallette,
            0xFF48 => self.sprite_pallete_1,
            0xFF49 => self.sprite_pallete_2,
            _ => { /*println!("Unsupported GPU read at {:#06x}", raw_address);*/ 0xFF }
        }
    }

    fn get_all_sprites (&self) -> Vec<Sprite> {
        let mut out = vec![];

        // There's room for 40 sprites in the OAM table
        for i in 0..40 {
            let address: u16 = i * 4;

            let y_pos = self.oam.read(address) as i32 - 16;
            let x_pos = self.oam.read(address + 1) as i32 - 8;
            let pattern_id = self.oam.read(address + 2);
            let attribs = self.oam.read(address + 3);

            let above_bg = (attribs & 0b1000_0000) != 0b1000_0000;
            let y_flip = (attribs & 0b0100_0000) == 0b0100_0000;
            let x_flip = (attribs & 0b0010_0000) == 0b0010_0000;
            let use_palette_0 = (attribs & 0b0001_0000) != 0b0001_0000;

            out.push(Sprite {
                y_pos, x_pos, pattern_id,
                above_bg, y_flip, x_flip, use_palette_0
            })
        }

        out
    }

    fn begin_dma(&mut self, source: u8) {
        // Really, we should be disabling access to anything but HRAM now,
        // but if the rom is nice then there shouldn't be an issue.
        if self.dma_cycles != 0 {
            // println!("INTERRUPTING DMA!")
        }
        self.dma_source = source;
        self.dma_cycles = gpu_timing::DMA_CYCLES;
    }

    fn update_dma (&mut self, ints: &mut Interrupts, mem: &mut Memory) {
        // There isn't one pending
        if self.dma_cycles == 0 { return; }

        self.dma_cycles -= 1;
        // Ready to actually perform DMA?
        if self.dma_cycles == 0 {
            let source = (self.dma_source as u16) * 0x100;

            for i in 0x00..=0x9F {
                let data = mem.read(ints, self, source + i);
                self.oam.write(i, data);
            }
        }
    }

    fn enter_vblank (&mut self, ints: &mut Interrupts) {
        ints.raise_interrupt(InterruptReason::VBlank);

        // TODO: This seems like odd behaviour to me.
        if self.status.vblank_interrupt {
            ints.raise_interrupt(InterruptReason::LCDStat);
        }

        self.finished_frame = self.frame.clone();
    }

    fn run_ly_compare (&mut self, ints: &mut Interrupts) {
        if self.ly == self.lyc {
            self.status.coincidence_flag = true;

            if self.status.lyc {
                ints.raise_interrupt(InterruptReason::LCDStat);
            }
        }
    }

    pub fn step(&mut self, ints: &mut Interrupts, mem: &mut Memory) {
        // TODO: Check that a DMA is performed even with display off
        self.update_dma(ints, mem);

        if !self.control.display_enable {
            return;
        }

        self.lx = (self.lx + 1) % gpu_timing::HTOTAL;

        let mode = self.status.get_mode();

        let new_mode = match mode {
            LcdMode::VBlank => {
                if self.lx == 0 {
                    self.ly = (self.ly + 1) % gpu_timing::VTOTAL;
                    self.run_ly_compare(ints);

                    if self.ly == 0 {
                        if self.status.oam_interrupt {
                            ints.raise_interrupt(InterruptReason::LCDStat);
                        }
                        LcdMode::OAMSearch
                    } else { mode }
                } else { mode }
            },
            _ => {
                match self.lx {
                    0 => {
                        self.ly += 1;
                        self.run_ly_compare(ints);
                        // Done with frame, enter VBlank
                        if self.ly == gpu_timing::VBLANK_ON {
                            self.enter_vblank(ints);
                            LcdMode::VBlank
                        } else { LcdMode::OAMSearch }
                    }
                    gpu_timing::HTRANSFER_ON => LcdMode::Transfer,
                    gpu_timing::HBLANK_ON => {
                        if self.status.hblank_interrupt {
                            ints.raise_interrupt(InterruptReason::LCDStat)
                        }
                        LcdMode::HBlank
                    },
                    _ => mode
                }
            }
        };

        if new_mode == LcdMode::OAMSearch && new_mode != self.status.get_mode() {
            // We've just entered OAMSearch, here we get the sprites
            self.sprite_cache = self.get_all_sprites();
        }

        self.status.set_mode(new_mode);

        // The first line takes longer to draw
        let line_start = gpu_timing::HTRANSFER_ON +
            if self.ly == 0 { 160 } else { 48 };

        if self.lx == line_start && self.status.get_mode() != LcdMode::VBlank {
            // Draw the current line
            // TODO: Move these draw_pixel calls into the mode switch above
            //       to allow mid-scanline visual effects
            let sprites = self.get_sprites_on_line(self.ly);
            for x in 0..(SCREEN_WIDTH as u8) {
                self.draw_pixel(ints, mem, &sprites, x, self.ly);
            }
        }
    }

    fn draw_pixel (&mut self, ints: &Interrupts, mem: &Memory, sprites_on_line: &Vec<Sprite>, x: u8, y: u8) {
        let ux = x as usize; let uy = y as usize;
        let idx = uy * SCREEN_WIDTH + ux;

        let bg_col: GreyShade;
        let bg_col_id = if self.control.bg_display {
            let id = self.get_background_colour_at(ints, mem, x, y);
            bg_col = self.get_shade_from_colour_id(id, self.bg_pallette);
            id
        } else {
            bg_col = GreyShade::White;
            0
        };

        // If there's a non-transparent sprite here, use its colour
        let s_col = self.get_sprite_colour_at(ints, mem, sprites_on_line, bg_col, bg_col_id, x, y);

        self.frame[idx] = s_col;
    }

    fn get_colour_id_in_line (&self, tile_line: u16, subx: u8) -> u16 {
        let lower = tile_line & 0xFF;
        let upper = (tile_line & 0xFF00) >> 8;

        let shift_amnt = 7 - subx;
        let mask = 1 << shift_amnt;
        let u = (upper & mask) >> shift_amnt;
        let l = (lower & mask) >> shift_amnt;
        let pixel_colour_id = (u << 1) | l;

        pixel_colour_id
    }

    fn get_shade_from_colour_id (&self, pixel_colour_id: u16, palette: u8) -> GreyShade {
        let shift_2 = pixel_colour_id * 2;
        let shade = (palette & (0b11 << shift_2)) >> shift_2;

        GreyShade::from(shade)
    }

    fn get_background_colour_at (&self, ints: &Interrupts, mem: &Memory, x: u8, y: u8) -> u16 {
        let is_window = self.control.window_enable &&
            x as isize > self.wx as isize - 7 && y >= self.wy;

        let tilemap_select = if is_window { 
            self.control.window_tile_map_display_select 
        } else {
            self.control.bg_tile_map_display_select
        };

        let tilemap_base = if tilemap_select {
            0x9C00
        } else { 0x9800 };

        // This is which tile ID our pixel is in
        let x16: u16;
        let y16: u16;

        if is_window {
            // TODO: This is a bit guessed but seems to work
            //       Try more games with windows
            x16 = x.wrapping_sub(self.wx - 7) as u16;
            y16 = y.wrapping_sub(self.wy) as u16;
        } else {
            x16 = x.wrapping_add(self.scx) as u16;
            y16 = y.wrapping_add(self.scy) as u16;
        }

        let tx = x16 / 8; let ty = y16 / 8;
        let subx = (x16 % 8) as u8; let suby = y16 % 8;

        let byte_offset = ty * 32 + tx;

        let tile_id_raw = mem.read(ints, self, tilemap_base + byte_offset);
        let tile_id: u16;

        if self.control.bg_and_window_data_select {
            // 0x8000 addressing mode
            tile_id = tile_id_raw as u16;
        } else {
            // 0x8800 addressing mode
            if tile_id_raw < 128 {
                tile_id = (tile_id_raw as u16) + 256;
            } else { tile_id = tile_id_raw as u16 }
        }

        let tile_byte_offset = tile_id * 16;
        let tile_line_offset = tile_byte_offset + (suby * 2);

        // This is the line of the tile data that out pixel resides on
        let tiledata_base = 0x8000;

        let tile_line = mem.read_16(ints, self, tiledata_base + tile_line_offset);
        let col_id = self.get_colour_id_in_line(tile_line, subx);
        col_id
    }

    fn get_sprite_colour_at (&self, ints: &Interrupts, mem: &Memory, sprites: &Vec<Sprite>, bg_col: GreyShade, bg_col_id: u16, x: u8, y: u8) -> GreyShade {
        // Sprites are hidden for this scanline
        if !self.control.obj_enable {
            return bg_col
        }

        let sprite_height = if self.control.obj_size { 16 } else { 8 };

        let ix = x as i32; let iy = y as i32;

        let mut maybe_colour: Option<GreyShade> = None;
        let mut min_x: i32 = SCREEN_WIDTH as i32 + 8;
        for sprite in sprites {
            if sprite.x_pos <= ix && (sprite.x_pos + 8) > ix && sprite.x_pos < min_x {
                if !sprite.above_bg && bg_col_id != 0 {
                    continue;
                }
        
                let mut subx = (ix - sprite.x_pos) as u8;
                let mut suby = iy - sprite.y_pos;
        
                // Tile address for 8x8 mode
                let mut pattern = sprite.pattern_id;
        
                // TODO: Might not be right
                if sprite_height == 16 {
                    if suby > 7 {
                        suby -= 7;
        
                        if sprite.y_flip {
                            pattern = sprite.pattern_id & 0xFE;
                        } else {
                            pattern = sprite.pattern_id | 0x01;
                        }
                    } else {
                        if sprite.y_flip {
                            pattern = sprite.pattern_id | 0x01;
                        } else {
                            pattern = sprite.pattern_id & 0xFE;
                        }
                    }
                }
        
                if sprite.x_flip { subx = 7 - subx }
                // TODO: Not sure if this applies to vertically flipped 8x16 mode sprites
                if sprite.y_flip { suby = 7 - suby }
        
                let tile_address = 0x8000 + (pattern as u16) * 16;
                let line_we_need = suby as u16 * 2;
                let tile_line = mem.read_16(ints, self, tile_address + line_we_need);
        
                let col_id = self.get_colour_id_in_line(tile_line, subx);
        
                if col_id == 0 {
                    // This pixel is transparent
                    continue
                } else {
                    let palette = if sprite.use_palette_0
                        { self.sprite_pallete_1 } else { self.sprite_pallete_2 };
                    
                    min_x = sprite.x_pos;
                    maybe_colour = Some(self.get_shade_from_colour_id(col_id, palette))
                }
            }
        }

        match maybe_colour {
            Some(col) => col,
            None => bg_col
        }
    }

    // Will be used later for get_sprite_pixel
    fn get_sprites_on_line (&self, y: u8) -> Vec<Sprite> {
        let sprite_height = if self.control.obj_size { 16 } else { 8 };

        let iy = y as i32;
        let mut on_line = vec![];
        for s in &self.sprite_cache {
            if s.y_pos <= iy && (s.y_pos + sprite_height) > iy {
                on_line.push(s.clone());
            }
        }

        on_line
    }

    pub fn get_sfml_frame (&self) -> [u8; SCREEN_RGBA_SLICE_SIZE] {
        let mut out_array = [0; SCREEN_RGBA_SLICE_SIZE];
        for i in 0..SCREEN_BUFFER_SIZE {
            let start = i * 4;
            match &self.finished_frame[i] {
                GreyShade::White => {
                    out_array[start] = 0xDD;
                    out_array[start + 1] = 0xDD;
                    out_array[start + 2] = 0xDD;
                    out_array[start + 3] = 0xFF;
                },
                GreyShade::LightGrey => {
                    out_array[start] = 0xAA;
                    out_array[start + 1] = 0xAA;
                    out_array[start + 2] = 0xAA;
                    out_array[start + 3] = 0xFF;
                },
                GreyShade::DarkGrey => {
                    out_array[start] = 0x88;
                    out_array[start + 1] = 0x88;
                    out_array[start + 2] = 0x88;
                    out_array[start + 3] = 0xFF;
                },
                GreyShade::Black => {
                    out_array[start] = 0x55;
                    out_array[start + 1] = 0x55;
                    out_array[start + 2] = 0x55;
                    out_array[start + 3] = 0xFF;
                }
            }
        }
        out_array
    }

    pub fn new () -> Gpu {
        let empty_frame = [GreyShade::White; SCREEN_BUFFER_SIZE];
        Gpu {
            frame: empty_frame,
            finished_frame: empty_frame.clone(),
            scy: 0, scx: 0, ly: 0, lx: 0, lyc:0, wy: 0, wx: 0,
            bg_pallette: 0, sprite_pallete_1: 0, sprite_pallete_2: 0,
            status: LcdStatus::new(),
            control: LcdControl::new(),
            oam: Ram::new(OAM_SIZE),
            dma_source: 0, dma_cycles: 0,
            sprite_cache: vec![]
        }
    }
}
