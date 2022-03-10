#![no_std]

extern crate alloc;

use {
    alloc::{
        format,
        boxed::Box
    },
    anyhow::Error,
    crankstart::{
        crankstart_game,
        geometry::{ScreenPoint, ScreenVector, ScreenRect},
        graphics::{Graphics, LCDColor, LCDSolidColor},
        system::System,
        Game, Playdate,
    },
    crankstart_sys::{LCD_COLUMNS, LCD_ROWS, PDButtons},
    euclid::{point2, vec2, size2},
};

mod gameboy;
use gameboy::cpu::Cpu;
use gameboy::lcd::GreyShade;
use gameboy::constants::{
    CYCLES_PER_FRAME, SCREEN_BUFFER_SIZE,
    SCREEN_WIDTH, SCREEN_HEIGHT
};

// We'll draw "gameboy pixels" at 4 "playdate pixels" wide each
const PIXEL_RATIO: usize = 2;

struct State {
    processor: Cpu
}

impl State {
    pub fn new(_playdate: &Playdate) -> Result<Box<Self>, Error> {
        crankstart::display::Display::get().set_refresh_rate(30.0)?;
        
        let mut cpu = Cpu::from_hardcoded_tetris();

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

        let mut cycles = 0;
        while cycles < CYCLES_PER_FRAME {
            cycles += self.processor.step();
        }

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
                    );
                }
            }
        }

        Ok(())
    }
}

crankstart_game!(State);
