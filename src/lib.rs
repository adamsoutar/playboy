#![no_std]

extern crate alloc;

use alloc::boxed::Box;
use anyhow::Error;
use crankstart::{
    crankstart_game,
    geometry::{ScreenRect},
    graphics::{Graphics, LCDColor, LCDSolidColor},
    system::System,
    Game, Playdate,
};
use crankstart_sys::{PDButtons, LCD_ROWS};
use euclid::{point2, size2, num::Floor};

use gbrs_core::{
    cpu::Cpu,
    lcd::GreyShade,
    constants::*
};

// The Playdate LCD actually updates at half the rate of the Gameboy
const FRAME_RATE: usize = 30;
// This is how much we'll scale the Gameboy screen to fit it on the Playdate
const SCALE_FACTOR: f32 = 1.6666666667;
// Start the image at this x coordinate (centers the scaled image)
const START_X: i32 = 67;

struct State {
    processor: Cpu,
    // This is used to determine when the crank has changed direction
    // (we use that for Start/Select)
    last_crank_change: f32
}

impl State {
    pub fn new(_playdate: &Playdate) -> Result<Box<Self>, Error> {
        crankstart::display::Display::get().set_refresh_rate(30.0)?;
        
        let mut cpu = Cpu::from_rom_bytes(
            include_bytes!("../rom.gb").to_vec()
        );
        cpu.frame_rate = FRAME_RATE;

        Ok(Box::new(Self {
            processor: cpu,
            last_crank_change: 0.
        }))
    }
}

// This is kind of like a differential.
// We're looking for a "change in change" in crank angle
fn process_crank_change (new_crank: f32, old_crank: f32) -> f32 {
    // Is this safe with floats? (no epsilon etc.)
    if old_crank > 0. && new_crank > 0. { 0. }
    else if old_crank < 0. && new_crank < 0. { 0. }
    else if new_crank == 0. { 0. }
    else { new_crank }
}

impl Game for State {
    fn update(&mut self, _playdate: &mut Playdate) -> Result<(), Error> {
        let system = System::get();
        let graphics = Graphics::get();

        graphics.clear(LCDColor::Solid(LCDSolidColor::kColorBlack))?;

        let crank_change = system.get_crank_change()?;
        let processed_crank = process_crank_change(crank_change, self.last_crank_change);
        self.last_crank_change = crank_change;

        let (btns_held, _, _) = system.get_button_state()?;

        // TODO: Raise the joypad interrupt
        self.processor.mem.joypad.a_pressed = (btns_held & PDButtons::kButtonA) == PDButtons::kButtonA;
        self.processor.mem.joypad.b_pressed = (btns_held & PDButtons::kButtonB) == PDButtons::kButtonB;
        self.processor.mem.joypad.up_pressed = (btns_held & PDButtons::kButtonUp) == PDButtons::kButtonUp;
        self.processor.mem.joypad.down_pressed = (btns_held & PDButtons::kButtonDown) == PDButtons::kButtonDown;
        self.processor.mem.joypad.left_pressed = (btns_held & PDButtons::kButtonLeft) == PDButtons::kButtonLeft;
        self.processor.mem.joypad.right_pressed = (btns_held & PDButtons::kButtonRight) == PDButtons::kButtonRight;
        self.processor.mem.joypad.start_pressed = processed_crank > 0.;
        self.processor.mem.joypad.select_pressed = processed_crank < 0.;

        // Actually *run* the Gameboy game.
        self.processor.step_one_frame();

        // Draw screen
        // TODO: While drawing 1x1 rects might work in the simulator, it may
        //   not be performant on-device.
        let playdate_x_pixels = (SCREEN_WIDTH as f32 * SCALE_FACTOR).floor() as usize;
        let playdate_y_pixels = LCD_ROWS as usize;

        for x in 0..playdate_x_pixels {
            for y in 0..playdate_y_pixels {
                let gameboy_x = (x as f32 / SCALE_FACTOR).floor() as usize;
                let gameboy_y = (y as f32 / SCALE_FACTOR).floor() as usize;
                let gameboy_lcd_index = gameboy_y * SCREEN_WIDTH + gameboy_x;
                let shade_at = &self.processor.gpu.finished_frame[gameboy_lcd_index];

                match shade_at {
                    // Not light enough to draw in 1-bit
                    GreyShade::Black => {},
                    GreyShade::DarkGrey => {},
                    _ => {
                        // Light enough!
                        graphics.fill_rect(
                            ScreenRect::new(point2(START_X + x as i32, y as i32), size2(1, 1)),
                            LCDColor::Solid(LCDSolidColor::kColorWhite)
                        )?;
                    }
                }
            }
        }

        Ok(())
    }
}

crankstart_game!(State);
