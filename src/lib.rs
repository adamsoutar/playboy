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
use crankstart_sys::PDButtons;
use euclid::{point2, size2};

use gbrs_core::{
    cpu::Cpu,
    lcd::GreyShade,
    constants::*
};

// We'll draw "gameboy pixels" at N "playdate pixels" wide each
const PIXEL_RATIO: usize = 2;
// The Playdate LCD actually updates at half the rate of the Gameboy
const FRAME_RATE: usize = 30;

struct State {
    processor: Cpu
}

impl State {
    pub fn new(_playdate: &Playdate) -> Result<Box<Self>, Error> {
        crankstart::display::Display::get().set_refresh_rate(30.0)?;
        
        let mut cpu = Cpu::from_rom_bytes(
            include_bytes!("../../gbrs/roms/Tetris.gb").to_vec()
        );
        cpu.frame_rate = FRAME_RATE;

        Ok(Box::new(Self {
            processor: cpu
        }))
    }
}

impl Game for State {
    fn update(&mut self, _playdate: &mut Playdate) -> Result<(), Error> {
        let graphics = Graphics::get();
        graphics.clear(LCDColor::Solid(LCDSolidColor::kColorWhite))?;

        let (btns_held, _, _) = System::get().get_button_state()?;

        // TODO: Raise the joypad interrupt
        // self.processor.mem.joypad.a_pressed = (btns_held & PDButtons::kButtonA) == PDButtons::kButtonA;
        self.processor.mem.joypad.b_pressed = (btns_held & PDButtons::kButtonB) == PDButtons::kButtonB;
        self.processor.mem.joypad.start_pressed = (btns_held & PDButtons::kButtonA) == PDButtons::kButtonA;
        // self.processor.mem.joypad.select_pressed = Key::is_pressed(Key::BackSpace);
        self.processor.mem.joypad.up_pressed = (btns_held & PDButtons::kButtonUp) == PDButtons::kButtonUp;
        self.processor.mem.joypad.down_pressed = (btns_held & PDButtons::kButtonDown) == PDButtons::kButtonDown;
        self.processor.mem.joypad.left_pressed = (btns_held & PDButtons::kButtonLeft) == PDButtons::kButtonLeft;
        self.processor.mem.joypad.right_pressed = (btns_held & PDButtons::kButtonRight) == PDButtons::kButtonRight;

        // Actually *run* the Gameboy game.
        self.processor.step_one_frame();

        // Draw screen
        // NOTE: This is not performant. We're just experimenting here.
        for i in 0..SCREEN_BUFFER_SIZE {
            let x = (i % SCREEN_WIDTH) * PIXEL_RATIO;
            let y = (i / SCREEN_WIDTH) * PIXEL_RATIO;
            let shade_at = &self.processor.gpu.finished_frame[i];

            match shade_at {
                // Not dark enough to draw in 1-bit
                GreyShade::White => {},
                GreyShade::LightGrey => {},
                _ => {
                    // Dark enough!
                    graphics.fill_rect(
                        ScreenRect::new(point2(x as i32, y as i32), size2(PIXEL_RATIO as i32, PIXEL_RATIO as i32)),
                        LCDColor::Solid(LCDSolidColor::kColorBlack)
                    ).unwrap();
                }
            }
        }

        Ok(())
    }
}

crankstart_game!(State);
